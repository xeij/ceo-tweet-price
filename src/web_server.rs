//! Web server for CEO Tweet Analyzer dashboard
//!
//! Serves a web UI and provides API endpoints for analysis data

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, services::ServeDir};

mod analysis;
mod models;
mod prolog;
mod storage;
mod stocks;
mod twitter;

use models::AnalysisResult;

/// CEO/Ticker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CeoConfig {
    ceo_handle: String,
    ticker: String,
    company: String,
}

/// Application state
#[derive(Clone)]
struct AppState {
    results: Arc<RwLock<Vec<AnalysisResult>>>,
    twitter_token: Option<String>,
    twitter_username: Option<String>,
    twitter_password: Option<String>,
    stock_api_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting CEO Tweet Analyzer Web Server...\n");

    // Get API keys from environment
    let twitter_token = std::env::var("TWITTER_BEARER_TOKEN").ok();
    let twitter_username = std::env::var("TWITTER_USERNAME").ok();
    let twitter_password = std::env::var("TWITTER_PASSWORD").ok();
    
    if twitter_token.is_none() && (twitter_username.is_none() || twitter_password.is_none()) {
        println!("WARNING: Neither TWITTER_BEARER_TOKEN nor TWITTER_USERNAME/PASSWORD are set.");
        println!("         Fetching tweets might fail.");
    }

    let stock_api_key = std::env::var("STOCK_API_KEY")
        .expect("STOCK_API_KEY environment variable not set");

    // Load existing results
    let cached_results = storage::load_results().unwrap_or_else(|e| {
        println!("No existing data found or failed to load: {}", e);
        Vec::new()
    });
    
    if !cached_results.is_empty() {
        println!("Loaded {} existing analysis results from disk", cached_results.len());
    }

    // Initialize app state
    let state = AppState {
        results: Arc::new(RwLock::new(cached_results)),
        twitter_token,
        twitter_username,
        twitter_password,
        stock_api_key,
    };

    // Build router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/analyze", post(run_analysis))
        .route("/api/results", get(get_results))
        .route("/api/status", get(get_status))
        .nest_service("/static", ServeDir::new("web/static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = "127.0.0.1:3000";
    println!("Server running at http://{}", addr);
    println!("Open your browser to view the dashboard\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve the main HTML page
async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../web/index.html"))
}

/// Get current analysis results
async fn get_results(State(state): State<AppState>) -> impl IntoResponse {
    let results = state.results.read().await;
    Json(results.clone())
}

/// Get server status
async fn get_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.1.0"
    }))
}

/// Run analysis for all configured CEOs
async fn run_analysis(State(state): State<AppState>) -> impl IntoResponse {
    println!("Starting batch analysis...");

    // Load CEO configuration
    let config_str = match std::fs::read_to_string("ceo_config.json") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: Failed to read ceo_config.json: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to read configuration" })),
            );
        }
    };

    let configs: Vec<CeoConfig> = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Failed to parse ceo_config.json: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to parse configuration" })),
            );
        }
    };

    println!("Loaded {} CEO/ticker pairs", configs.len());

    let mut results = Vec::new();
    let days = 90; // this will be used for price history, but tweet count is strictly limited in twitter.rs

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
            state.twitter_token.as_deref(),
            state.twitter_username.as_deref(),
            state.twitter_password.as_deref(),
            days,
            false,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                eprintln!("    WARNING: Failed to fetch tweets: {}", e);
                // Continue to next CEO on error
                continue;
            }
        };

        if tweets.is_empty() {
            println!("    WARNING: No tweets found");
            continue;
        }

        // Fetch stock prices
        let prices = match stocks::fetch_prices(&config.ticker, &state.stock_api_key, days, false)
            .await
        {
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

        // Rate limiting: wait between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Store results
    let mut state_results = state.results.write().await;
    *state_results = results.clone();

    // Save to disk
    if let Err(e) = storage::save_results(&results) {
        eprintln!("ERROR: Failed to save results to disk: {}", e);
    } else {
        println!("Saved analysis results to data/results.json");
    }

    println!("\nBatch analysis complete! Analyzed {} companies\n", results.len());

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "analyzed": results.len(),
        "message": format!("Successfully analyzed {} companies", results.len())
    })))
}
