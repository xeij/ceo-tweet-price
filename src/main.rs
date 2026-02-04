//! CEO Tweet Analyzer
//!
//! This application fetches CEO tweets, retrieves stock price data,
//! and analyzes correlations between tweet sentiment and stock movements.
//! It uses Prolog for rule-based pattern detection and Lean 4 for formal verification.

mod cli;
mod models;
mod twitter;
mod stocks;
mod analysis;
mod prolog;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Cli::parse();
    
    // Validate arguments
    args.validate()?;
    
    // Set up logging based on verbosity
    if args.verbose {
        println!("Running in verbose mode");
        println!("CEO Handle: @{}", args.ceo_handle);
        println!("Stock Ticker: {}", args.ticker);
        println!("Days to analyze: {}", args.days);
    }
    
    println!("\nCEO Tweet Analyzer Starting...\n");
    
    // Step 1: Fetch tweets
    println!("Fetching tweets from @{}...", args.ceo_handle);
    let tweets = twitter::fetch_tweets(
        &args.ceo_handle,
        &args.api_key_twitter,
        args.days,
        args.verbose,
    )
    .await?;
    
    println!("Fetched {} tweets", tweets.len());
    
    // Step 2: Fetch stock prices
    println!("\nFetching stock prices for {}...", args.ticker);
    let prices = stocks::fetch_prices(
        &args.ticker,
        &args.api_key_stocks,
        args.days,
        args.verbose,
    )
    .await?;
    
    println!("Fetched {} price points", prices.len());
    
    // Step 3: Perform analysis
    println!("\nAnalyzing tweet impacts and correlations...");
    let mut analysis_result = analysis::analyze(
        &args.ceo_handle,
        &args.ticker,
        tweets,
        prices,
        args.verbose,
    )?;
    
    println!("Analysis complete");
    
    // Step 4: Apply Prolog rules
    println!("\nApplying Prolog rules for pattern detection...");
    prolog::apply_rules(&mut analysis_result, args.export_prolog.as_deref())?;
    
    println!("Prolog analysis complete");
    
    // Step 5: Display results
    println!("\nResults:\n");
    display_results(&analysis_result, &args)?;
    
    // Step 6: Generate chart if requested
    if let Some(chart_path) = &args.chart_output {
        println!("\nGenerating chart to {}...", chart_path);
        // TODO: Implement chart generation with plotters
        println!("WARNING: Chart generation not yet implemented");
    }
    
    println!("\nAnalysis complete!\n");
    
    Ok(())
}

/// Display analysis results based on output format
fn display_results(result: &models::AnalysisResult, args: &Cli) -> Result<()> {
    use cli::OutputFormat;
    
    match args.output_format {
        OutputFormat::Table | OutputFormat::Both => {
            display_table(result)?;
        }
        _ => {}
    }
    
    if matches!(args.output_format, OutputFormat::Json | OutputFormat::Both) {
        display_json(result)?;
    }
    
    Ok(())
}

/// Display results as a formatted table
fn display_table(result: &models::AnalysisResult) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  CEO Tweet Impact Analysis");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  CEO: @{}", result.ceo_handle);
    println!("  Ticker: {}", result.ticker);
    println!("  Period: {} to {}", 
             result.start_date.format("%Y-%m-%d"),
             result.end_date.format("%Y-%m-%d"));
    println!("  Total Tweets: {}", result.total_tweets);
    println!("  Tweets with Price Data: {}", result.tweets_with_price_data);
    println!("═══════════════════════════════════════════════════════════════════════════\n");
    
    // Summary statistics
    println!("Summary Statistics:");
    println!("  Correlation (sentiment vs 1d change): {:.4}", 
             result.correlation_1d.unwrap_or(0.0));
    println!("  Correlation (sentiment vs 3d change): {:.4}", 
             result.correlation_3d.unwrap_or(0.0));
    println!("  Positive tweets → >3% rise (1d): {:.1}%", 
             result.positive_tweets_with_rise_1d);
    println!("  Positive tweets → >3% rise (3d): {:.1}%", 
             result.positive_tweets_with_rise_3d);
    
    // Top impactful tweets
    println!("\nMost Impactful Tweets (by Prolog rules):");
    let impactful: Vec<_> = result.impacts.iter()
        .filter(|i| i.is_impactful)
        .take(5)
        .collect();
    
    if impactful.is_empty() {
        println!("  No tweets classified as impactful");
    } else {
        for (idx, impact) in impactful.iter().enumerate() {
            let text = if impact.tweet.text.len() > 60 {
                format!("{}...", &impact.tweet.text[..60])
            } else {
                impact.tweet.text.clone()
            };
            
            println!("\n  {}. {} ({})", 
                     idx + 1,
                     impact.tweet.created_at.format("%Y-%m-%d"),
                     text);
            println!("     Sentiment: {:.2} | 1d: {:+.2}% | 3d: {:+.2}%",
                     impact.tweet.sentiment.unwrap_or(0.0),
                     impact.change_1d.unwrap_or(0.0),
                     impact.change_3d.unwrap_or(0.0));
        }
    }
    
    println!("\n═══════════════════════════════════════════════════════════════════════════\n");
    
    Ok(())
}

/// Display results as JSON
fn display_json(result: &models::AnalysisResult) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{}", json);
    Ok(())
}
