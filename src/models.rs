//! Core data models for the CEO tweet analyzer.
//!
//! This module defines the primary data structures used throughout the application,
//! including tweets, stock prices, and analysis results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single tweet from a CEO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    /// Unique tweet ID
    pub id: String,
    
    /// Tweet text content
    pub text: String,
    
    /// When the tweet was created
    pub created_at: DateTime<Utc>,
    
    /// Number of retweets
    pub retweet_count: u32,
    
    /// Number of likes
    pub like_count: u32,
    
    /// Calculated sentiment score (-1.0 to 1.0)
    /// Negative = bearish, Positive = bullish, 0 = neutral
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<f64>,
}

/// Represents a stock price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Stock ticker symbol
    pub ticker: String,
    
    /// Date of this price point
    pub date: DateTime<Utc>,
    
    /// Opening price
    pub open: f64,
    
    /// Closing price
    pub close: f64,
    
    /// Highest price during the day
    pub high: f64,
    
    /// Lowest price during the day
    pub low: f64,
    
    /// Trading volume
    pub volume: u64,
}

impl PricePoint {
    /// Calculate the percentage change from open to close
    pub fn daily_change_percent(&self) -> f64 {
        if self.open == 0.0 {
            0.0
        } else {
            ((self.close - self.open) / self.open) * 100.0
        }
    }
}

/// Represents the analysis of a single tweet's impact on stock price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetImpact {
    /// The tweet being analyzed
    pub tweet: Tweet,
    
    /// Stock price on the day of the tweet
    pub price_at_tweet: Option<f64>,
    
    /// Percentage change 1 day after tweet
    pub change_1d: Option<f64>,
    
    /// Percentage change 3 days after tweet
    pub change_3d: Option<f64>,
    
    /// Whether this tweet is classified as "impactful" by Prolog rules
    pub is_impactful: bool,
}

/// Overall analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// CEO handle analyzed
    pub ceo_handle: String,
    
    /// Stock ticker analyzed
    pub ticker: String,
    
    /// Date range of analysis
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    
    /// All tweet impacts
    pub impacts: Vec<TweetImpact>,
    
    /// Pearson correlation coefficient between sentiment and 1-day price change
    pub correlation_1d: Option<f64>,
    
    /// Pearson correlation coefficient between sentiment and 3-day price change
    pub correlation_3d: Option<f64>,
    
    /// Percentage of positive tweets followed by >3% rise (1 day)
    pub positive_tweets_with_rise_1d: f64,
    
    /// Percentage of positive tweets followed by >3% rise (3 days)
    pub positive_tweets_with_rise_3d: f64,
    
    /// Total number of tweets analyzed
    pub total_tweets: usize,
    
    /// Number of tweets with available price data
    pub tweets_with_price_data: usize,
}

impl AnalysisResult {
    /// Create a new empty analysis result
    pub fn new(ceo_handle: String, ticker: String, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Self {
        Self {
            ceo_handle,
            ticker,
            start_date,
            end_date,
            impacts: Vec::new(),
            correlation_1d: None,
            correlation_3d: None,
            positive_tweets_with_rise_1d: 0.0,
            positive_tweets_with_rise_3d: 0.0,
            total_tweets: 0,
            tweets_with_price_data: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_point_daily_change() {
        let price = PricePoint {
            ticker: "TSLA".to_string(),
            date: Utc::now(),
            open: 100.0,
            close: 110.0,
            high: 115.0,
            low: 95.0,
            volume: 1000000,
        };
        
        assert_eq!(price.daily_change_percent(), 10.0);
    }

    #[test]
    fn test_price_point_zero_open() {
        let price = PricePoint {
            ticker: "TSLA".to_string(),
            date: Utc::now(),
            open: 0.0,
            close: 110.0,
            high: 115.0,
            low: 0.0,
            volume: 1000000,
        };
        
        assert_eq!(price.daily_change_percent(), 0.0);
    }
}
