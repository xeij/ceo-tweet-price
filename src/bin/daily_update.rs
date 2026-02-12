//! Daily update script for CI/CD
//!
//! Fetches stock prices from Yahoo Finance (free, no API key)
//! and tweet counts from Twitter.
//! Tracks MONTHLY metrics - resets at the start of each month.

use anyhow::{Context, Result};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

/// CEO/Ticker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CeoConfig {
    ceo_handle: String,
    ticker: String,
    company: String,
}

/// Tracking data for a single CEO/stock pair (MONTHLY)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingEntry {
    ticker: String,
    company: String,
    ceo_handle: String,

    // Current month being tracked (e.g., "2026-02")
    current_month: String,

    // Price at the start of the month
    month_start_price: f64,

    // Current price
    current_price: f64,

    // Monthly price change (%)
    monthly_price_change_pct: f64,
    price_direction: String, // "up", "down", "flat"

    // Tweet tracking (THIS MONTH)
    tweets_this_month: u32,
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
    current_month: String,
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
#[allow(dead_code)]
struct YahooError {
    code: String,
    description: String,
}

fn get_current_month() -> String {
    Utc::now().format("%Y-%m").to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CEO Tweet Tracker - Monthly Update ===\n");

    let current_month = get_current_month();
    println!("Current month: {}", current_month);

    // Load CEO configuration
    let config_str = std::fs::read_to_string("ceo_config.json")
        .context("Failed to read ceo_config.json")?;
    let configs: Vec<CeoConfig> = serde_json::from_str(&config_str)
        .context("Failed to parse ceo_config.json")?;

    println!("Loaded {} CEO/ticker pairs", configs.len());

    // Load or create tracking database
    let mut db = load_or_create_database(&configs, &current_month)?;

    // Check if we need to reset for a new month
    if db.current_month != current_month {
        println!("\nNEW MONTH detected! Resetting monthly tracking...");
        reset_for_new_month(&mut db, &current_month);
    }

    println!("Tracking month: {}\n", db.current_month);

    // Update each entry
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    println!("Using Gemini API for AI-powered tweet counting\n");

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
        let month_start_price = entry.month_start_price;

        // Fetch current stock price from Yahoo Finance
        match fetch_yahoo_price(&client, &ticker).await {
            Ok(price) => {
                let entry = &mut db.entries[idx];
                entry.current_price = price;

                if month_start_price > 0.0 {
                    // Calculate monthly change
                    entry.monthly_price_change_pct = ((price - month_start_price) / month_start_price) * 100.0;
                    entry.price_direction = if entry.monthly_price_change_pct > 0.5 {
                        "up".to_string()
                    } else if entry.monthly_price_change_pct < -0.5 {
                        "down".to_string()
                    } else {
                        "flat".to_string()
                    };
                } else {
                    // First update this month - set start price
                    entry.month_start_price = price;
                    entry.monthly_price_change_pct = 0.0;
                    entry.price_direction = "flat".to_string();
                }
                print!("${:.2} ({:+.2}%) ", price, entry.monthly_price_change_pct);
            }
            Err(e) => {
                print!("price error: {} ", e);
            }
        }

        // Fetch tweet count using Gemini API (Direct REST)
        match fetch_tweet_count(&ceo_handle, &client).await {
            Ok((total, positive, negative, neutral)) => {
                let entry = &mut db.entries[idx];
                entry.tweets_this_month = total;
                entry.positive_tweets = positive;
                entry.negative_tweets = negative;
                entry.neutral_tweets = neutral;
                println!("tweets: {} OK", total);
            }
            Err(e) => {
                println!("tweets: ERR ({})", e);
            }
        }
        
        // Add delay to avoid rate limits (Genesis/Gemini free tier)
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        db.entries[idx].last_updated = Utc::now().to_rfc3339();
        println!("OK");
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    }

    // Save database
    db.last_updated = Utc::now().to_rfc3339();
    save_database(&db)?;

    println!("\n=== Update complete! ===");
    println!("Data saved to data/tracking.json");
    println!("Month: {} | Entries: {}", db.current_month, db.entries.len());

    Ok(())
}

fn load_or_create_database(configs: &[CeoConfig], current_month: &str) -> Result<TrackingDatabase> {
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
        current_month: current_month.to_string(),
        month_start_price: 0.0,
        current_price: 0.0,
        monthly_price_change_pct: 0.0,
        price_direction: "flat".to_string(),
        tweets_this_month: 0,
        positive_tweets: 0,
        negative_tweets: 0,
        neutral_tweets: 0,
        last_updated: now.to_rfc3339(),
    }).collect();

    Ok(TrackingDatabase {
        created_at: now.to_rfc3339(),
        last_updated: now.to_rfc3339(),
        current_month: current_month.to_string(),
        entries,
    })
}

/// Reset all entries for a new month
fn reset_for_new_month(db: &mut TrackingDatabase, new_month: &str) {
    db.current_month = new_month.to_string();

    for entry in &mut db.entries {
        entry.current_month = new_month.to_string();
        // Keep current_price as the new month's start price
        entry.month_start_price = entry.current_price;
        entry.monthly_price_change_pct = 0.0;
        entry.price_direction = "flat".to_string();
        entry.tweets_this_month = 0;
        entry.positive_tweets = 0;
        entry.negative_tweets = 0;
        entry.neutral_tweets = 0;
    }
}

fn save_database(db: &TrackingDatabase) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let json = serde_json::to_string_pretty(db)?;
    std::fs::write("data/tracking.json", json)?;
    Ok(())
}

/// Fetch stock price from Yahoo Finance (no API key needed)
async fn fetch_yahoo_price(client: &reqwest::Client, ticker: &str) -> Result<f64> {
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

use serde_json::json;

/// Fetch tweet count using Gemini API (Direct REST)
async fn fetch_tweet_count(handle: &str, client: &reqwest::Client) -> Result<(u32, u32, u32, u32)> {
    // Get Gemini API key from environment
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            // No API key set, return zeros
            return Ok((0, 0, 0, 0));
        }
    };
    
    let now = Utc::now();
    let month_name = now.format("%B").to_string(); // e.g., "February"
    let year = now.year();
    
    // Ask Gemini about tweet count
    let prompt = format!(
        "How many tweets did @{} post on Twitter/X in {} {}? \
         Please reply with ONLY a number, nothing else. \
         If you cannot find this information, reply with 0.",
        handle, month_name, year
    );
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite:generateContent?key={}", 
        api_key
    );
    
    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });
    
    let response = client.post(&url)
        .json(&body)
        .send()
        .await
        .context("Failed to call Gemini API")?;
        
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        eprintln!("Gemini API error for @{}: {} - {}", handle, status, text);
        return Ok((0, 0, 0, 0));
    }
    
    let json_resp: serde_json::Value = response.json().await
        .context("Failed to parse Gemini response")?;
        
    // Extract text from: candidates[0].content.parts[0].text
    if let Some(text) = json_resp["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        // Try to parse the first number found in the response
        let count = text
            .split_whitespace()
            .find_map(|word| word.trim().parse::<u32>().ok())
            .unwrap_or(0);
        
        return Ok((count, 0, 0, count));
    }
    
    Ok((0, 0, 0, 0))
}

/// Simple keyword-based sentiment analysis
fn analyze_sentiment(text: &str) -> f64 {
    let text_lower = text.to_lowercase();

    let positive_words = [
        "great", "excellent", "amazing", "good", "success", "win", "winning",
        "growth", "profit", "record", "best", "excited", "love", "fantastic",
        "incredible", "revolutionary", "breakthrough", "proud", "happy",
        "bullish", "moon", "rocket", "innovation", "strong", "opportunity",
    ];

    let negative_words = [
        "bad", "terrible", "awful", "poor", "loss", "losing", "fail", "failure",
        "worst", "sad", "disappointed", "concern", "problem", "issue", "difficult",
        "challenge", "unfortunate", "regret", "sorry", "bearish", "crash", "down",
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

    score
}
