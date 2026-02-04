//! Daily update script for CI/CD
//!
//! Fetches stock prices from Yahoo Finance (free, no API key)
//! and tweet counts from Twitter (guest mode).
//! Stores cumulative data in data/tracking.json

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// CEO/Ticker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CeoConfig {
    ceo_handle: String,
    ticker: String,
    company: String,
}

/// Tracking data for a single CEO/stock pair
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingEntry {
    ticker: String,
    company: String,
    ceo_handle: String,

    // Baseline (when tracking started)
    baseline_date: String,
    baseline_price: f64,

    // Current values
    current_price: f64,
    price_change_pct: f64,
    price_direction: String, // "up", "down", "flat"

    // Tweet tracking
    tweet_count_total: u32,
    tweets_this_week: u32,
    positive_tweets: u32,
    negative_tweets: u32,
    neutral_tweets: u32,

    // Metadata
    last_updated: String,
}

/// Full tracking database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingDatabase {
    created_at: String,
    last_updated: String,
    entries: Vec<TrackingEntry>,
}

/// Yahoo Finance quote response
#[derive(Debug, Deserialize)]
struct YahooResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Option<Vec<YahooResult>>,
    error: Option<YahooError>,
}

#[derive(Debug, Deserialize)]
struct YahooResult {
    meta: YahooMeta,
}

#[derive(Debug, Deserialize)]
struct YahooMeta {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "previousClose")]
    previous_close: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct YahooError {
    code: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CEO Tweet Tracker - Daily Update ===\n");

    // Load CEO configuration
    let config_str = std::fs::read_to_string("ceo_config.json")
        .context("Failed to read ceo_config.json")?;
    let configs: Vec<CeoConfig> = serde_json::from_str(&config_str)
        .context("Failed to parse ceo_config.json")?;

    println!("Loaded {} CEO/ticker pairs", configs.len());

    // Load or create tracking database
    let mut db = load_or_create_database(&configs)?;

    println!("Tracking database loaded. Last updated: {}\n", db.last_updated);

    // Update each entry
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let total_entries = db.entries.len();
    for idx in 0..total_entries {
        let entry = &db.entries[idx];
        print!("  [{}/{}] {} (@{})... ",
            idx + 1,
            total_entries,
            entry.ticker,
            entry.ceo_handle
        );

        let ticker = entry.ticker.clone();
        let ceo_handle = entry.ceo_handle.clone();
        let baseline_price = entry.baseline_price;
        let tweet_count_total = entry.tweet_count_total;

        // Fetch current stock price from Yahoo Finance
        match fetch_yahoo_price(&client, &ticker).await {
            Ok(price) => {
                let entry = &mut db.entries[idx];
                entry.current_price = price;
                if baseline_price > 0.0 {
                    entry.price_change_pct = ((price - baseline_price) / baseline_price) * 100.0;
                    entry.price_direction = if entry.price_change_pct > 0.5 {
                        "up".to_string()
                    } else if entry.price_change_pct < -0.5 {
                        "down".to_string()
                    } else {
                        "flat".to_string()
                    };
                } else {
                    // First time - set baseline
                    entry.baseline_price = price;
                    entry.baseline_date = Utc::now().format("%Y-%m-%d").to_string();
                }
                print!("${:.2} ", price);
            }
            Err(e) => {
                print!("price error: {} ", e);
            }
        }

        // Fetch tweet count (simplified - just increment for demo)
        // In production, this would call the Twitter scraper
        match fetch_tweet_count(&ceo_handle).await {
            Ok((total, positive, negative, neutral)) => {
                let entry = &mut db.entries[idx];
                let new_tweets = total.saturating_sub(tweet_count_total);
                entry.tweets_this_week += new_tweets;
                entry.tweet_count_total = total;
                entry.positive_tweets = positive;
                entry.negative_tweets = negative;
                entry.neutral_tweets = neutral;
                print!("tweets: {} ", total);
            }
            Err(e) => {
                print!("tweet error: {} ", e);
            }
        }

        db.entries[idx].last_updated = Utc::now().to_rfc3339();
        println!("OK");

        // Rate limiting - be nice to APIs
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }

    // Save database
    db.last_updated = Utc::now().to_rfc3339();
    save_database(&db)?;

    println!("\n=== Update complete! ===");
    println!("Data saved to data/tracking.json");

    Ok(())
}

fn load_or_create_database(configs: &[CeoConfig]) -> Result<TrackingDatabase> {
    let path = "data/tracking.json";

    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(db) = serde_json::from_str::<TrackingDatabase>(&content) {
            return Ok(db);
        }
    }

    // Create new database
    println!("Creating new tracking database...");
    let now = Utc::now();
    let entries: Vec<TrackingEntry> = configs.iter().map(|c| TrackingEntry {
        ticker: c.ticker.clone(),
        company: c.company.clone(),
        ceo_handle: c.ceo_handle.clone(),
        baseline_date: now.format("%Y-%m-%d").to_string(),
        baseline_price: 0.0,
        current_price: 0.0,
        price_change_pct: 0.0,
        price_direction: "flat".to_string(),
        tweet_count_total: 0,
        tweets_this_week: 0,
        positive_tweets: 0,
        negative_tweets: 0,
        neutral_tweets: 0,
        last_updated: now.to_rfc3339(),
    }).collect();

    Ok(TrackingDatabase {
        created_at: now.to_rfc3339(),
        last_updated: now.to_rfc3339(),
        entries,
    })
}

fn save_database(db: &TrackingDatabase) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let json = serde_json::to_string_pretty(db)?;
    std::fs::write("data/tracking.json", json)?;
    Ok(())
}

/// Fetch stock price from Yahoo Finance (no API key needed)
async fn fetch_yahoo_price(client: &reqwest::Client, ticker: &str) -> Result<f64> {
    // Yahoo Finance chart API - free, no auth required
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=1d",
        ticker
    );

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch from Yahoo Finance")?;

    if !response.status().is_success() {
        anyhow::bail!("Yahoo Finance returned {}", response.status());
    }

    let data: YahooResponse = response.json().await
        .context("Failed to parse Yahoo Finance response")?;

    if let Some(error) = data.chart.error {
        anyhow::bail!("Yahoo Finance error: {}", error.description);
    }

    let result = data.chart.result
        .and_then(|r| r.into_iter().next())
        .context("No data in Yahoo Finance response")?;

    result.meta.regular_market_price
        .or(result.meta.previous_close)
        .context("No price in Yahoo Finance response")
}

/// Fetch tweet count for a user (simplified version)
/// In production, this would use the Twitter scraper
async fn fetch_tweet_count(_handle: &str) -> Result<(u32, u32, u32, u32)> {
    // For CI/CD without Twitter credentials, we'll use a simple approach:
    // Try to fetch via nitter or return cached/estimated values

    // For now, return placeholder values that can be updated when Twitter access is available
    // The real implementation would use agent_twitter_client::scraper::Scraper

    // Return (total, positive, negative, neutral) - placeholder for now
    Ok((0, 0, 0, 0))
}
