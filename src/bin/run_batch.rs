#[path = "../analysis.rs"]
mod analysis;
#[path = "../models.rs"]
mod models;
#[path = "../prolog.rs"]
mod prolog;
#[path = "../stocks.rs"]
mod stocks;
#[path = "../storage.rs"]
mod storage;
#[path = "../twitter.rs"]
mod twitter;

use anyhow::Result;
use models::AnalysisResult;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct CeoConfig {
    ceo_handle: String,
    ticker: String,
    #[allow(dead_code)]
    company: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting CEO Tweet Analyzer Batch Runner...");

    // Get API keys
    let twitter_token = std::env::var("TWITTER_BEARER_TOKEN").ok();
    let twitter_username = std::env::var("TWITTER_USERNAME").ok();
    let twitter_password = std::env::var("TWITTER_PASSWORD").ok();
    
    if twitter_token.is_none() && (twitter_username.is_none() || twitter_password.is_none()) {
         println!("WARNING: No Twitter credentials found (API token or username/password).");
    }

    let stock_api_key = std::env::var("STOCK_API_KEY")
        .expect("STOCK_API_KEY environment variable not set");

    // Load configuration
    // ... (lines 38-48 match existing, skipping for brevity in replacement if possible, but I must replace contiguous block)
    let config_str = std::fs::read_to_string("ceo_config.json")
        .expect("Failed to read ceo_config.json");
    let configs: Vec<CeoConfig> = serde_json::from_str(&config_str)
        .expect("Failed to parse configurations");

    println!("Loaded {} CEO/ticker pairs", configs.len());

    let mut results = Vec::new();
    let days = 90;

    // Process each CEO (limit to first 25)
    for (idx, config) in configs.iter().take(25).enumerate() {
        println!(
            "  [{}/25] Analyzing @{} / {}...",
            idx + 1,
            config.ceo_handle,
            config.ticker
        );

        // Fetch tweets
        let tweets = match twitter::fetch_tweets(
            &config.ceo_handle,
            twitter_token.as_deref(),
            twitter_username.as_deref(),
            twitter_password.as_deref(),
            days,
            false,
        ).await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("    WARNING: Failed to fetch tweets: {}", e);
                continue;
            }
        };

        if tweets.is_empty() {
            println!("    WARNING: No tweets found");
            continue;
        }

        // Fetch stock prices
        let prices = match stocks::fetch_prices(
            &config.ticker,
            &stock_api_key,
            days,
            false,
        ).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("    WARNING: Failed to fetch prices: {}", e);
                continue;
            }
        };

        if prices.is_empty() {
            println!("    WARNING: No price data found");
            continue;
        }

        // Analyze
        let mut result = match analysis::analyze(
            &config.ceo_handle,
            &config.ticker,
            tweets,
            prices,
            false,
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("    WARNING: Analysis failed: {}", e);
                continue;
            }
        };

        // Apply Prolog rules
        if let Err(e) = prolog::apply_rules(&mut result, None) {
            eprintln!("    WARNING: Prolog rules failed: {}", e);
        }

        println!(
            "    SUCCESS: Correlation: {:.3}, Tweets: {}",
            result.correlation_1d.unwrap_or(0.0),
            result.total_tweets
        );

        results.push(result);

        // Rate limiting
        sleep(Duration::from_millis(500)).await;
    }

    println!("\nBatch analysis complete! Analyzed {} companies", results.len());

    // Save results
    if !results.is_empty() {
        storage::save_results(&results)?;
        println!("Saved analysis results to data/results.json");
    } else {
        println!("No results to save.");
    }

    Ok(())
}
