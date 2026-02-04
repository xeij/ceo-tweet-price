//! Web server for CEO Tweet Tracker dashboard
//!
//! Serves a web UI showing tracked CEO tweets and stock prices.
//! Data is updated daily via CI/CD and stored in data/tracking.json

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

/// Tracking data for a single CEO/stock pair
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingEntry {
    ticker: String,
    company: String,
    ceo_handle: String,
    baseline_date: String,
    baseline_price: f64,
    current_price: f64,
    price_change_pct: f64,
    price_direction: String,
    tweet_count_total: u32,
    tweets_this_week: u32,
    positive_tweets: u32,
    negative_tweets: u32,
    neutral_tweets: u32,
    last_updated: String,
}

/// Full tracking database
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingDatabase {
    created_at: String,
    last_updated: String,
    entries: Vec<TrackingEntry>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting CEO Tweet Tracker Web Server...\n");

    // Build router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/data", get(get_tracking_data))
        .route("/api/status", get(get_status))
        .layer(CorsLayer::permissive());

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

/// Get tracking data from JSON file
async fn get_tracking_data() -> impl IntoResponse {
    match std::fs::read_to_string("data/tracking.json") {
        Ok(content) => {
            match serde_json::from_str::<TrackingDatabase>(&content) {
                Ok(db) => (StatusCode::OK, Json(serde_json::json!({
                    "success": true,
                    "created_at": db.created_at,
                    "last_updated": db.last_updated,
                    "entries": db.entries
                }))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to parse tracking data: {}", e)
                }))),
            }
        }
        Err(_) => {
            // Return empty data if file doesn't exist yet
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "created_at": null,
                "last_updated": null,
                "entries": []
            })))
        }
    }
}

/// Get server status
async fn get_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.2.0"
    }))
}
