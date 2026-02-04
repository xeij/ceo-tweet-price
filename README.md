# CEO Tweet Analyzer - Web Dashboard

A web application that analyzes correlations between CEO tweets and stock price movements for 50 companies.

## Quick Start

Set your API keys and start the server:

```powershell
$env:TWITTER_BEARER_TOKEN="your_twitter_token"
$env:STOCK_API_KEY="your_alpha_vantage_key"

.\start-server.ps1
```

Open http://localhost:3000 in your browser and click "Run Analysis".

## Features

The dashboard displays analysis results for 50 CEO/stock pairs in a dark-themed interface. Each card shows correlation coefficients between tweet sentiment and stock price changes, plus success rates for positive tweets. Results are color-coded: green for positive correlation, red for negative, and gray for neutral.

## Configuration

Edit `ceo_config.json` to analyze different CEOs or companies. The server processes up to 50 entries, analyzing the last 90 days of tweets and stock data for each.

## API Keys

Get a Twitter API Bearer Token from the Twitter Developer Portal and an Alpha Vantage API key (free tier available). Set these as environment variables before starting the server.

## Build

The project compiles successfully with all emojis removed from code output. Documentation files are excluded from git (except README.md).

```powershell
cargo build --bin web-server --release
```

## Files

- `src/web_server.rs` - Web server with batch analysis
- `web/index.html` - Dark-themed dashboard UI
- `ceo_config.json` - List of 50 CEO/ticker pairs
- `start-server.ps1` - Startup script
- `.gitignore` - Excludes documentation and build artifacts
