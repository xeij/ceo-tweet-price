//! Prolog integration for rule-based pattern detection.
//!
//! This module generates Prolog facts from analysis results and applies
//! declarative rules to identify impactful tweets.

use crate::models::AnalysisResult;
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;

/// Apply Prolog rules to identify impactful tweets
///
/// # Arguments
/// * `result` - Analysis result to process (will be modified in place)
/// * `export_path` - Optional path to export Prolog facts
pub fn apply_rules(result: &mut AnalysisResult, export_path: Option<&str>) -> Result<()> {
    // Generate Prolog facts
    let facts = generate_facts(result);
    
    // Export if requested
    if let Some(path) = export_path {
        let mut file = fs::File::create(path)
            .context(format!("Failed to create Prolog export file: {}", path))?;
        
        file.write_all(facts.as_bytes())
            .context("Failed to write Prolog facts")?;
        
        println!("  → Exported Prolog facts to {}", path);
    }
    
    // Apply rules using scryer-prolog
    // Note: This is a simplified version. Full implementation would use scryer-prolog crate
    // to actually query the facts. For now, we'll use a simple Rust-based rule engine.
    apply_simple_rules(result);
    
    Ok(())
}

/// Generate Prolog facts from analysis results
fn generate_facts(result: &AnalysisResult) -> String {
    let mut facts = String::new();
    
    // Header comment
    facts.push_str(&format!(
        "% Prolog facts for CEO Tweet Analysis\n\
         % CEO: @{}\n\
         % Ticker: {}\n\
         % Generated: {}\n\n",
        result.ceo_handle,
        result.ticker,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    ));
    
    // Define predicates
    facts.push_str(
        "% tweet(TweetId, Date, Sentiment, Retweets, Likes).\n\
         % price_change(TweetId, Days, PercentChange).\n\
         % impactful_tweet(TweetId) :- ...\n\n"
    );
    
    // Generate facts for each tweet impact
    for (idx, impact) in result.impacts.iter().enumerate() {
        let tweet_id = format!("tweet_{}", idx);
        let date = impact.tweet.created_at.format("%Y%m%d");
        let sentiment = impact.tweet.sentiment.unwrap_or(0.0);
        
        // Tweet fact
        facts.push_str(&format!(
            "tweet('{}', {}, {:.3}, {}, {}).\n",
            tweet_id,
            date,
            sentiment,
            impact.tweet.retweet_count,
            impact.tweet.like_count
        ));
        
        // Price change facts
        if let Some(change_1d) = impact.change_1d {
            facts.push_str(&format!(
                "price_change('{}', 1, {:.3}).\n",
                tweet_id, change_1d
            ));
        }
        
        if let Some(change_3d) = impact.change_3d {
            facts.push_str(&format!(
                "price_change('{}', 3, {:.3}).\n",
                tweet_id, change_3d
            ));
        }
    }
    
    // Add rules
    facts.push_str("\n% Rules for identifying impactful tweets\n");
    facts.push_str(
        "% A tweet is impactful if:\n\
         % 1. It has strong sentiment (|sentiment| > 0.3) AND\n\
         % 2. It caused significant price movement (|change| > 3%) within 1-3 days\n\n"
    );
    
    facts.push_str(
        "impactful_tweet(TweetId) :-\n\
         \ttweet(TweetId, _, Sentiment, _, _),\n\
         \tabs(Sentiment) > 0.3,\n\
         \tprice_change(TweetId, Days, Change),\n\
         \tDays =< 3,\n\
         \tabs(Change) > 3.0.\n\n"
    );
    
    facts.push_str(
        "highly_impactful_tweet(TweetId) :-\n\
         \ttweet(TweetId, _, Sentiment, _, _),\n\
         \tabs(Sentiment) > 0.5,\n\
         \tprice_change(TweetId, Days, Change),\n\
         \tDays =< 3,\n\
         \tabs(Change) > 5.0.\n\n"
    );
    
    facts.push_str(
        "viral_impactful_tweet(TweetId) :-\n\
         \ttweet(TweetId, _, Sentiment, Retweets, Likes),\n\
         \tRetweets > 10000,\n\
         \tLikes > 50000,\n\
         \timpactful_tweet(TweetId).\n"
    );
    
    facts
}

/// Apply simple rule-based logic to mark impactful tweets
/// This is a Rust implementation of the Prolog rules for demonstration
fn apply_simple_rules(result: &mut AnalysisResult) {
    for impact in &mut result.impacts {
        let sentiment = impact.tweet.sentiment.unwrap_or(0.0);
        
        // Rule: Strong sentiment + significant price movement
        let has_strong_sentiment = sentiment.abs() > 0.3;
        
        let has_significant_movement = impact
            .change_1d
            .map(|c| c.abs() > 3.0)
            .unwrap_or(false)
            || impact
                .change_3d
                .map(|c| c.abs() > 3.0)
                .unwrap_or(false);
        
        impact.is_impactful = has_strong_sentiment && has_significant_movement;
    }
    
    // Sort impacts by "impactfulness" (impactful first, then by sentiment strength)
    result.impacts.sort_by(|a, b| {
        match (a.is_impactful, b.is_impactful) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_sent = a.tweet.sentiment.unwrap_or(0.0).abs();
                let b_sent = b.tweet.sentiment.unwrap_or(0.0).abs();
                b_sent.partial_cmp(&a_sent).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Tweet, TweetImpact};
    use chrono::Utc;

    #[test]
    fn test_generate_facts() {
        let mut result = AnalysisResult::new(
            "elonmusk".to_string(),
            "TSLA".to_string(),
            Utc::now(),
            Utc::now(),
        );
        
        result.impacts.push(TweetImpact {
            tweet: Tweet {
                id: "123".to_string(),
                text: "Great news!".to_string(),
                created_at: Utc::now(),
                retweet_count: 1000,
                like_count: 5000,
                sentiment: Some(0.8),
            },
            price_at_tweet: Some(100.0),
            change_1d: Some(5.0),
            change_3d: Some(7.0),
            is_impactful: false,
        });
        
        let facts = generate_facts(&result);
        
        assert!(facts.contains("tweet("));
        assert!(facts.contains("price_change("));
        assert!(facts.contains("impactful_tweet("));
    }
}
