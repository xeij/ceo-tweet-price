//! Web server for CEO Tweet Tracker dashboard
//!
//! Serves a web UI showing tracked CEO tweets and stock prices.
//! Data is updated daily via CI/CD and stored in data/tracking.json
//! Tracks MONTHLY metrics - tweets this month and stock change since month start.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

/// Tracking data for a single CEO/stock pair (MONTHLY)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrackingEntry {
    ticker: String,
    company: String,
    ceo_handle: String,
    current_month: String,
    month_start_price: f64,
    current_price: f64,
    monthly_price_change_pct: f64,
    price_direction: String,
    tweets_this_month: u32,
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
    current_month: String,
    entries: Vec<TrackingEntry>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting CEO Tweet Tracker Web Server...\n");

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/data", get(get_tracking_data))
        .route("/api/status", get(get_status))
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:3000";
    println!("Server running at http://{}", addr);
    println!("Open your browser to view the dashboard\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../web/index.html"))
}

async fn get_tracking_data() -> impl IntoResponse {
    match std::fs::read_to_string("data/tracking.json") {
        Ok(content) => {
            match serde_json::from_str::<TrackingDatabase>(&content) {
                Ok(db) => (StatusCode::OK, Json(serde_json::json!({
                    "success": true,
                    "created_at": db.created_at,
                    "last_updated": db.last_updated,
                    "current_month": db.current_month,
                    "entries": db.entries
                }))),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to parse tracking data: {}", e)
                }))),
            }
        }
        Err(_) => {
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "created_at": null,
                "last_updated": null,
                "current_month": null,
                "entries": []
            })))
        }
    }
}

async fn get_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.3.0"
    }))
}
