//! Analysis module for correlating tweets with stock price movements.
//!
//! This module handles:
//! - Sentiment analysis of tweets
//! - Alignment of tweets with stock prices
//! - Calculation of price changes after tweets
//! - Statistical correlation analysis

use crate::models::{AnalysisResult, PricePoint, Tweet, TweetImpact};
use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Perform complete analysis of tweets and stock prices
pub fn analyze(
    ceo_handle: &str,
    ticker: &str,
    mut tweets: Vec<Tweet>,
    prices: Vec<PricePoint>,
    verbose: bool,
) -> Result<AnalysisResult> {
    if verbose {
        println!("  → Calculating sentiment for {} tweets...", tweets.len());
    }
    
    // Step 1: Calculate sentiment for all tweets
    for tweet in &mut tweets {
        tweet.sentiment = Some(calculate_sentiment(&tweet.text));
    }
    
    if verbose {
        println!("  → Aligning tweets with price data...");
    }
    
    // Step 2: Create price lookup map by date
    let price_map = create_price_map(&prices);
    
    // Step 3: Calculate impacts for each tweet
    let mut impacts = Vec::new();
    let mut tweets_with_data = 0;
    
    for tweet in &tweets {
        let impact = calculate_tweet_impact(tweet, &price_map);
        
        if impact.price_at_tweet.is_some() {
            tweets_with_data += 1;
        }
        
        impacts.push(impact);
    }
    
    if verbose {
        println!("  → Calculating correlations...");
    }
    
    // Step 4: Calculate correlations
    let correlation_1d = calculate_correlation(&impacts, |i| i.change_1d);
    let correlation_3d = calculate_correlation(&impacts, |i| i.change_3d);
    
    // Step 5: Calculate positive tweet success rates
    let (pos_rise_1d, pos_rise_3d) = calculate_positive_tweet_stats(&impacts);

    // Step 6: Calculate tweet counts
    let positive_tweets = tweets.iter().filter(|t| t.sentiment.unwrap_or(0.0) > 0.0).count();
    let negative_tweets = tweets.iter().filter(|t| t.sentiment.unwrap_or(0.0) < 0.0).count();
    let neutral_tweets = tweets.iter().filter(|t| t.sentiment.unwrap_or(0.0) == 0.0).count();
    
    // Step 7: Calculate stock performance
    let performance_1w = calculate_period_performance(&prices, 7);
    let performance_1m = calculate_period_performance(&prices, 30);
    let performance_3m = calculate_period_performance(&prices, 90);
    
    // Step 8: Build result
    let start_date = tweets.iter().map(|t| t.created_at).min().unwrap_or(Utc::now());
    let end_date = tweets.iter().map(|t| t.created_at).max().unwrap_or(Utc::now());
    
    let mut result = AnalysisResult::new(
        ceo_handle.to_string(),
        ticker.to_string(),
        start_date,
        end_date,
    );
    
    result.impacts = impacts;
    result.correlation_1d = correlation_1d;
    result.correlation_3d = correlation_3d;
    result.positive_tweets_with_rise_1d = pos_rise_1d;
    result.positive_tweets_with_rise_3d = pos_rise_3d;
    result.performance_1w = performance_1w;
    result.performance_1m = performance_1m;
    result.performance_3m = performance_3m;
    result.positive_tweets = positive_tweets;
    result.negative_tweets = negative_tweets;
    result.neutral_tweets = neutral_tweets;
    result.total_tweets = tweets.len();
    result.tweets_with_price_data = tweets_with_data;
    
    Ok(result)
}

/// Calculate stock performance over a specific period of days
fn calculate_period_performance(prices: &[PricePoint], days: i64) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }

    // Find latest price (end of period)
    let latest = prices.iter().max_by_key(|p| p.date)?;
    
    // Target date in the past
    let target_date = latest.date - Duration::days(days);
    
    // Find closest price at or before target date
    // We want the price roughly 'days' ago. If exact date missing, use closest previous.
    let past_price = prices.iter()
        .filter(|p| p.date <= target_date)
        .max_by_key(|p| p.date);

    match past_price {
        Some(past) => {
            if past.close == 0.0 {
                None
            } else {
                Some(((latest.close - past.close) / past.close) * 100.0)
            }
        },
        None => None // Not enough data history
    }
}

/// Calculate sentiment score for tweet text using keyword-based approach
///
/// Returns a score between -1.0 (very negative) and 1.0 (very positive)
fn calculate_sentiment(text: &str) -> f64 {
    let text_lower = text.to_lowercase();
    
    // Simple keyword lists (can be expanded)
    let positive_words = [
        "great", "excellent", "amazing", "good", "success", "win", "winning",
        "growth", "profit", "record", "best", "excited", "love", "fantastic",
        "incredible", "revolutionary", "breakthrough", "proud", "happy",
    ];
    
    let negative_words = [
        "bad", "terrible", "awful", "poor", "loss", "losing", "fail", "failure",
        "worst", "sad", "disappointed", "concern", "problem", "issue", "difficult",
        "challenge", "unfortunate", "regret", "sorry",
    ];
    
    let mut score = 0.0;
    
    for word in &positive_words {
        if text_lower.contains(word) {
            score += 1.0;
        }
    }
    
    for word in &negative_words {
        if text_lower.contains(word) {
            score -= 1.0;
        }
    }
    
    // Normalize to [-1, 1] range
    let max_score = positive_words.len().max(negative_words.len()) as f64;
    if max_score > 0.0 {
        score = score / max_score;
    }
    
    score.clamp(-1.0, 1.0)
}

/// Create a hashmap of prices indexed by date (YYYY-MM-DD)
fn create_price_map(prices: &[PricePoint]) -> HashMap<String, &PricePoint> {
    prices
        .iter()
        .map(|p| (p.date.format("%Y-%m-%d").to_string(), p))
        .collect()
}

/// Calculate the impact of a single tweet on stock prices
fn calculate_tweet_impact(tweet: &Tweet, price_map: &HashMap<String, &PricePoint>) -> TweetImpact {
    let tweet_date = tweet.created_at.format("%Y-%m-%d").to_string();
    
    // Get price at tweet date
    let price_at_tweet = price_map.get(&tweet_date).map(|p| p.close);
    
    // Calculate 1-day change
    let date_1d = (tweet.created_at + Duration::days(1)).format("%Y-%m-%d").to_string();
    let change_1d = if let (Some(base_price), Some(future_price)) = 
        (price_map.get(&tweet_date), price_map.get(&date_1d)) {
        Some(((future_price.close - base_price.close) / base_price.close) * 100.0)
    } else {
        None
    };
    
    // Calculate 3-day change
    let date_3d = (tweet.created_at + Duration::days(3)).format("%Y-%m-%d").to_string();
    let change_3d = if let (Some(base_price), Some(future_price)) = 
        (price_map.get(&tweet_date), price_map.get(&date_3d)) {
        Some(((future_price.close - base_price.close) / base_price.close) * 100.0)
    } else {
        None
    };
    
    TweetImpact {
        tweet: tweet.clone(),
        price_at_tweet,
        change_1d,
        change_3d,
        is_impactful: false, // Will be set by Prolog rules
    }
}

/// Calculate Pearson correlation between sentiment and price changes
fn calculate_correlation<F>(impacts: &[TweetImpact], get_change: F) -> Option<f64>
where
    F: Fn(&TweetImpact) -> Option<f64>,
{
    // Collect pairs of (sentiment, price_change) where both are available
    let pairs: Vec<(f64, f64)> = impacts
        .iter()
        .filter_map(|impact| {
            let sentiment = impact.tweet.sentiment?;
            let change = get_change(impact)?;
            Some((sentiment, change))
        })
        .collect();
    
    if pairs.len() < 2 {
        return None;
    }
    
    // Calculate means
    let n = pairs.len() as f64;
    let mean_x: f64 = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y: f64 = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;
    
    // Calculate correlation
    let mut numerator = 0.0;
    let mut sum_sq_x = 0.0;
    let mut sum_sq_y = 0.0;
    
    for (x, y) in &pairs {
        let dx = x - mean_x;
        let dy = y - mean_y;
        numerator += dx * dy;
        sum_sq_x += dx * dx;
        sum_sq_y += dy * dy;
    }
    
    let denominator = (sum_sq_x * sum_sq_y).sqrt();
    
    if denominator == 0.0 {
        return None;
    }
    
    Some(numerator / denominator)
}

/// Calculate percentage of positive tweets followed by >3% rise
fn calculate_positive_tweet_stats(impacts: &[TweetImpact]) -> (f64, f64) {
    let positive_tweets: Vec<_> = impacts
        .iter()
        .filter(|i| i.tweet.sentiment.unwrap_or(0.0) > 0.0)
        .collect();
    
    if positive_tweets.is_empty() {
        return (0.0, 0.0);
    }
    
    let count_1d = positive_tweets
        .iter()
        .filter(|i| i.change_1d.unwrap_or(0.0) > 3.0)
        .count();
    
    let count_3d = positive_tweets
        .iter()
        .filter(|i| i.change_3d.unwrap_or(0.0) > 3.0)
        .count();
    
    let total = positive_tweets.len() as f64;
    
    (
        (count_1d as f64 / total) * 100.0,
        (count_3d as f64 / total) * 100.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_positive() {
        let text = "This is great and amazing!";
        let score = calculate_sentiment(text);
        assert!(score > 0.0);
    }

    #[test]
    fn test_sentiment_negative() {
        let text = "This is terrible and awful!";
        let score = calculate_sentiment(text);
        assert!(score < 0.0);
    }

    #[test]
    fn test_sentiment_neutral() {
        let text = "This is a statement.";
        let score = calculate_sentiment(text);
        assert_eq!(score, 0.0);
    }
}
