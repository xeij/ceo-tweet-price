# CEO Tweet Analyzer

A command-line tool that analyzes correlations between CEO tweets and stock price movements using Rust, Prolog, and Lean 4 for formal verification.

## What It Does

This application fetches recent tweets from a CEO's Twitter account and historical stock prices for their company, then analyzes whether tweet sentiment correlates with stock price changes. It uses keyword-based sentiment analysis to score tweets from -1 (very negative) to +1 (very positive), then calculates the Pearson correlation between sentiment and price movements 1-3 days after each tweet. The tool also applies Prolog rules to identify "impactful" tweets—those with strong sentiment that coincided with significant price movements.

The project includes formal verification using Lean 4, with mathematical proofs ensuring the percentage change formula and correlation calculations are mathematically sound. This gives you confidence that the statistical analysis is correct by design, not just by testing.

## Quick Start

First, build the project with `cargo build --release`. You'll need API keys from Twitter (get a Bearer Token from the Twitter Developer Portal) and Alpha Vantage (free tier available). Set these as environment variables: `TWITTER_BEARER_TOKEN` and `STOCK_API_KEY`. Then run an analysis with `cargo run --release -- --ceo-handle elonmusk --ticker TSLA --days 90 --verbose`.

The output shows correlation statistics, the percentage of positive tweets followed by price increases, and the top impactful tweets identified by Prolog rules. You can export results as JSON with `--output-format json` or save Prolog facts with `--export-prolog facts.pl` for further querying.

## Example Usage

Analyze Elon Musk's tweets and Tesla stock: `cargo run --release -- --ceo-handle elonmusk --ticker TSLA --days 365`

Export Prolog facts for pattern analysis: `cargo run --release -- --ceo-handle elonmusk --ticker TSLA --days 90 --export-prolog facts.pl`

Get JSON output for programmatic processing: `cargo run --release -- --ceo-handle elonmusk --ticker TSLA --days 60 --output-format json > results.json`

Try different CEO/stock pairs like Tim Cook and Apple (`--ceo-handle tim_cook --ticker AAPL`) or Satya Nadella and Microsoft (`--ceo-handle satyanadella --ticker MSFT`).

## Understanding Results

Correlation values range from -1 to +1. Values between +0.3 and +1.0 indicate positive correlation (positive tweets tend to precede price increases), while -1.0 to -0.3 indicates negative correlation. Values between -0.3 and +0.3 suggest weak or no correlation. The tool also reports what percentage of positive tweets were followed by price rises greater than 3%, giving you a practical measure of predictive power.

A tweet is marked "impactful" if it has strong sentiment (magnitude > 0.3) AND caused significant price movement (> 3%) within 1-3 days. These are identified using declarative Prolog rules that you can customize in `src/prolog.rs`.

## Documentation

For detailed setup instructions and troubleshooting, see `QUICKSTART.md`. Developers should read `DEVELOPMENT.md` for architecture details, testing guidelines, and the project roadmap. The `PROJECT_SUMMARY.md` file provides a comprehensive overview of all components and design decisions.

## Technical Details

The application is built with modern Rust (2021 edition) using async/await for efficient API calls. It integrates with Twitter API v2 for tweet fetching and Alpha Vantage for stock data. The analysis engine implements Pearson correlation from scratch and uses a keyword-based sentiment analyzer that you can extend. Prolog integration generates facts and applies declarative rules for pattern detection. The Lean 4 proofs in the `lean/` directory formally verify that percentage changes and correlations behave correctly for all valid inputs.

## Troubleshooting

If you get a 401 error from Twitter, your Bearer Token is invalid—regenerate it in the Developer Portal. If Alpha Vantage reports rate limits, note that the free tier allows 5 calls per minute and 500 per day. If no tweets are found, verify the handle is correct (without the @ symbol) and that the account has public tweets. For build issues on Windows, try `cargo clean` and rebuild if you encounter file locking errors.

## License

MIT License - see LICENSE file for details.
