//! CLI argument parsing using clap.

use clap::Parser;

/// CEO Tweet Analyzer - Correlate CEO tweets with stock price movements
#[derive(Parser, Debug)]
#[command(name = "ceo-tweet-analyzer")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// CEO's Twitter/X handle (without @)
    #[arg(long, value_name = "HANDLE")]
    pub ceo_handle: String,

    /// Stock ticker symbol (e.g., TSLA, AAPL)
    #[arg(long, value_name = "TICKER")]
    pub ticker: String,

    /// Number of days to analyze (looking back from today)
    #[arg(long, value_name = "DAYS", default_value = "365")]
    pub days: u32,

    /// Twitter API Bearer Token (or set TWITTER_BEARER_TOKEN env var)
    #[arg(long, env = "TWITTER_BEARER_TOKEN")]
    pub api_key_twitter: String,

    /// Stock API Key (Alpha Vantage or Twelve Data, or set STOCK_API_KEY env var)
    #[arg(long, env = "STOCK_API_KEY")]
    pub api_key_stocks: String,

    /// Output format: table, json, or both
    #[arg(long, value_name = "FORMAT", default_value = "table")]
    pub output_format: String,

    /// Save results to JSON file
    #[arg(long, value_name = "FILE")]
    pub output_file: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}
