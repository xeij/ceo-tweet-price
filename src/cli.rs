//! Command-line interface definitions using clap.
//!
//! This module defines the CLI arguments for the CEO tweet analyzer,
//! including Twitter handle, stock ticker, date range, and API credentials.

use clap::Parser;

/// CEO Tweet Analyzer - Correlate CEO tweets with stock price movements
#[derive(Parser, Debug)]
#[command(
    name = "ceo-tweet-analyzer",
    version = "0.1.0",
    about = "Analyze CEO tweets and their correlation with stock price changes",
    long_about = "Fetches tweets from a CEO's Twitter account, retrieves corresponding \
                  stock price data, and performs correlation analysis using Rust, Prolog, \
                  and Lean 4 for formal verification."
)]
pub struct Cli {
    /// Twitter handle of the CEO (without @)
    #[arg(long, value_name = "HANDLE")]
    pub ceo_handle: String,

    /// Stock ticker symbol (e.g., TSLA, AAPL)
    #[arg(long, value_name = "TICKER")]
    pub ticker: String,

    /// Number of days to look back for tweets and stock data
    #[arg(long, default_value = "365", value_name = "DAYS")]
    pub days: u32,

    /// Twitter API Bearer Token (optional if using scraping)
    #[arg(long, env = "TWITTER_BEARER_TOKEN", value_name = "TOKEN")]
    pub api_key_twitter: Option<String>,

    /// Twitter Username (for scraping)
    #[arg(long, env = "TWITTER_USERNAME")]
    pub twitter_username: Option<String>,

    /// Twitter Password (for scraping)
    #[arg(long, env = "TWITTER_PASSWORD")]
    pub twitter_password: Option<String>,

    /// Stock API key (Alpha Vantage) (or set via STOCK_API_KEY env var)
    #[arg(long, env = "STOCK_API_KEY", value_name = "KEY")]
    pub api_key_stocks: String,

    /// Output format: table, json, or both
    #[arg(long, default_value = "table", value_name = "FORMAT")]
    pub output_format: OutputFormat,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Export Prolog facts to file
    #[arg(long, value_name = "PATH")]
    pub export_prolog: Option<String>,

    /// Generate chart (PNG file)
    #[arg(long, value_name = "PATH")]
    pub chart_output: Option<String>,
}

/// Output format options
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table
    Table,
    /// JSON output
    Json,
    /// Both table and JSON
    Both,
}

impl Cli {
    /// Validate CLI arguments
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.ceo_handle.is_empty() {
            anyhow::bail!("CEO handle cannot be empty");
        }
        
        if self.ticker.is_empty() {
            anyhow::bail!("Stock ticker cannot be empty");
        }
        
        if self.days == 0 || self.days > 3650 {
            anyhow::bail!("Days must be between 1 and 3650 (10 years)");
        }
        
        if self.api_key_twitter.is_empty() {
            anyhow::bail!("Twitter API key is required (use --api-key-twitter or TWITTER_BEARER_TOKEN env var)");
        }
        
        if self.api_key_stocks.is_empty() {
            anyhow::bail!("Stock API key is required (use --api-key-stocks or STOCK_API_KEY env var)");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_validation_empty_handle() {
        let cli = Cli {
            ceo_handle: String::new(),
            ticker: "TSLA".to_string(),
            days: 365,
            api_key_twitter: "test".to_string(),
            api_key_stocks: "test".to_string(),
            output_format: OutputFormat::Table,
            verbose: false,
            export_prolog: None,
            chart_output: None,
        };
        
        assert!(cli.validate().is_err());
    }

    #[test]
    fn test_cli_validation_valid() {
        let cli = Cli {
            ceo_handle: "elonmusk".to_string(),
            ticker: "TSLA".to_string(),
            days: 365,
            api_key_twitter: "test_token".to_string(),
            api_key_stocks: "test_key".to_string(),
            output_format: OutputFormat::Table,
            verbose: false,
            export_prolog: None,
            chart_output: None,
        };
        
        assert!(cli.validate().is_ok());
    }
}
