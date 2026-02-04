//! Stock price data fetching from Alpha Vantage API.
//!
//! This module retrieves historical daily stock prices for correlation analysis.

use crate::models::PricePoint;
use anyhow::{Context, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use std::collections::HashMap;

/// Alpha Vantage API base URL
const ALPHA_VANTAGE_BASE: &str = "https://www.alphavantage.co/query";

/// Response from Alpha Vantage TIME_SERIES_DAILY endpoint
#[derive(Debug, Deserialize)]
struct TimeSeriesResponse {
    #[serde(rename = "Time Series (Daily)")]
    time_series: Option<HashMap<String, DailyData>>,
    #[serde(rename = "Error Message")]
    error_message: Option<String>,
    #[serde(rename = "Note")]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

/// Fetch historical stock prices
///
/// # Arguments
/// * `ticker` - Stock ticker symbol (e.g., "TSLA")
/// * `api_key` - Alpha Vantage API key
/// * `days` - Number of days to look back (note: API returns up to 100 days for free tier)
/// * `verbose` - Enable verbose logging
///
/// # Returns
/// Vector of price points ordered by date (oldest first)
pub async fn fetch_prices(
    ticker: &str,
    api_key: &str,
    days: u32,
    verbose: bool,
) -> Result<Vec<PricePoint>> {
    if verbose {
        println!("  → Fetching daily prices for {}", ticker);
    }
    
    let client = reqwest::Client::new();
    
    // Alpha Vantage TIME_SERIES_DAILY endpoint
    // Note: Free tier gives last 100 days. For more, need premium or TIME_SERIES_DAILY_ADJUSTED with outputsize=full
    let url = format!(
        "{}?function=TIME_SERIES_DAILY&symbol={}&apikey={}&outputsize=compact",
        ALPHA_VANTAGE_BASE, ticker, api_key
    );
    
    if verbose {
        println!("  → Requesting data from Alpha Vantage...");
    }
    
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch stock data from Alpha Vantage")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Alpha Vantage API error ({}): {}", status, body);
    }
    
    let ts_response: TimeSeriesResponse = response
        .json()
        .await
        .context("Failed to parse Alpha Vantage response")?;
    
    // Check for API errors
    if let Some(error) = ts_response.error_message {
        anyhow::bail!("Alpha Vantage error: {}", error);
    }
    
    if let Some(note) = ts_response.note {
        if note.contains("API call frequency") {
            anyhow::bail!("Alpha Vantage rate limit exceeded: {}", note);
        }
    }
    
    let time_series = ts_response
        .time_series
        .context("No time series data in response")?;
    
    if verbose {
        println!("  → Parsing {} daily records...", time_series.len());
    }
    
    // Parse and convert to PricePoint structs
    let mut prices = Vec::new();
    
    for (date_str, daily_data) in time_series {
        // Parse date
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .context(format!("Failed to parse date: {}", date_str))?;
        let datetime = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
        
        // Parse price values
        let open = daily_data.open.parse::<f64>()
            .context(format!("Failed to parse open price: {}", daily_data.open))?;
        let high = daily_data.high.parse::<f64>()
            .context(format!("Failed to parse high price: {}", daily_data.high))?;
        let low = daily_data.low.parse::<f64>()
            .context(format!("Failed to parse low price: {}", daily_data.low))?;
        let close = daily_data.close.parse::<f64>()
            .context(format!("Failed to parse close price: {}", daily_data.close))?;
        let volume = daily_data.volume.parse::<u64>()
            .context(format!("Failed to parse volume: {}", daily_data.volume))?;
        
        prices.push(PricePoint {
            ticker: ticker.to_string(),
            date: datetime,
            open,
            close,
            high,
            low,
            volume,
        });
    }
    
    // Sort by date (oldest first)
    prices.sort_by(|a, b| a.date.cmp(&b.date));
    
    // Limit to requested days
    if prices.len() > days as usize {
        prices = prices.into_iter().rev().take(days as usize).rev().collect();
    }
    
    if verbose {
        println!("  → Returning {} price points", prices.len());
    }
    
    Ok(prices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_vantage_base_url() {
        assert_eq!(ALPHA_VANTAGE_BASE, "https://www.alphavantage.co/query");
    }
}
