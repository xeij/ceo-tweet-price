//! Data models for tweets, stock prices, and analysis results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single tweet from a CEO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
}

/// Represents a single stock price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Sentiment classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

impl Sentiment {
    /// Convert sentiment to numeric score for correlation analysis
    pub fn to_score(&self) -> f64 {
        match self {
            Sentiment::Positive => 1.0,
            Sentiment::Neutral => 0.0,
            Sentiment::Negative => -1.0,
        }
    }
}

/// Aligned data: tweet with corresponding stock price changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignedData {
    pub tweet: Tweet,
    pub sentiment: Sentiment,
    pub price_before: f64,
    pub price_1d_after: Option<f64>,
    pub price_3d_after: Option<f64>,
    pub delta_1d: Option<f64>, // Percentage change
    pub delta_3d: Option<f64>, // Percentage change
}

impl AlignedData {
    /// Calculate percentage change
    /// This is the formula we'll verify in Lean 4
    pub fn calculate_delta(before: f64, after: f64) -> f64 {
        ((after - before) / before) * 100.0
    }
}

/// Summary statistics for the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub total_tweets: usize,
    pub positive_tweets: usize,
    pub negative_tweets: usize,
    pub neutral_tweets: usize,
    pub avg_delta_1d: f64,
    pub avg_delta_3d: f64,
    pub correlation_sentiment_1d: Option<f64>,
    pub correlation_sentiment_3d: Option<f64>,
    pub impactful_tweets: Vec<String>, // Tweet IDs from Prolog
}
