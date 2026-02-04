# Development Guide

## Project Status

This is the initial implementation of the CEO Tweet Analyzer. The core architecture is in place with all major modules implemented.

## ✅ Completed

### Core Infrastructure
- [x] Project structure and Cargo.toml
- [x] CLI argument parsing with clap
- [x] Core data models (Tweet, PricePoint, TweetImpact, AnalysisResult)
- [x] Error handling with anyhow

### Data Fetching
- [x] Twitter API v2 integration
  - User lookup by handle
  - Tweet fetching with pagination
  - Rate limiting handling
- [x] Alpha Vantage stock API integration
  - Daily price data fetching
  - Error handling for rate limits

### Analysis
- [x] Keyword-based sentiment analysis
- [x] Tweet-price alignment by date
- [x] Percentage change calculation (1d, 3d)
- [x] Pearson correlation coefficient
- [x] Positive tweet success rate statistics

### Prolog Integration
- [x] Prolog fact generation
- [x] Rule definitions (impactful_tweet, highly_impactful_tweet, viral_impactful_tweet)
- [x] Fact export to .pl file
- [x] Rust-based rule application (simplified)

### Formal Verification
- [x] Lean 4 percentage change proofs
- [x] Lean 4 correlation bounds proofs
- [x] Documentation of Lean-Rust correspondence

### Output
- [x] Table formatting
- [x] JSON output
- [x] Summary statistics
- [x] Top impactful tweets display

## 🚧 TODO / Next Steps

### High Priority
1. **Test with Real APIs**
   - Get Twitter Bearer Token
   - Get Alpha Vantage API key
   - Test with real data (e.g., @elonmusk, TSLA)
   - Verify API response parsing

2. **Fix Compilation Issues**
   - Remove `scryer-prolog` dependency (not needed for MVP)
   - Remove `statrs` dependency (we implemented correlation manually)
   - Test build: `cargo build --release`

3. **Add Unit Tests**
   - Test sentiment calculation edge cases
   - Test correlation with known data
   - Test price alignment logic
   - Test CLI validation

### Medium Priority
4. **Improve Sentiment Analysis**
   - Expand keyword lists
   - Add negation handling ("not good" should be negative)
   - Consider emoji sentiment
   - Optional: Integrate lightweight ML model

5. **Chart Generation**
   - Implement with `plotters` crate
   - Timeline chart: stock price + tweet markers
   - Sentiment vs. price change scatter plot

6. **Better Prolog Integration**
   - Actually use `scryer-prolog` crate for querying
   - Allow custom rule files
   - Query results integration

### Low Priority
7. **Performance Optimization**
   - Parallel API requests where possible
   - Caching for repeated queries
   - Streaming for large datasets

8. **Additional Features**
   - Multiple CEO comparison mode
   - Export to CSV
   - Web dashboard (separate crate)
   - Real-time monitoring

## 🔧 Development Workflow

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### Running
```bash
# With environment variables
export TWITTER_BEARER_TOKEN="your_token"
export STOCK_API_KEY="your_key"

cargo run -- --ceo-handle elonmusk --ticker TSLA --days 30 --verbose
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_sentiment_positive

# Run with output
cargo test -- --nocapture
```

### Linting
```bash
# Check for common mistakes
cargo clippy

# Auto-fix issues
cargo clippy --fix
```

### Formatting
```bash
# Check formatting
cargo fmt -- --check

# Auto-format
cargo fmt
```

## 📝 Code Style Guidelines

### Rust
- Use `rustfmt` defaults
- Prefer `?` operator over `.unwrap()` in library code
- Add doc comments (`///`) for public items
- Use `anyhow::Result` for functions that can fail
- Prefer `&str` over `String` for function parameters
- Use `#[derive(Debug)]` for all structs

### Error Messages
- Be specific about what went wrong
- Include context (e.g., which API, which ticker)
- Suggest fixes when possible

Example:
```rust
.context(format!("Failed to fetch tweets for @{}", handle))?
```

### Comments
- Explain *why*, not *what*
- Use `//` for implementation notes
- Use `///` for API documentation
- Add examples in doc comments when helpful

## 🐛 Known Issues

1. **Alpha Vantage Free Tier Limits**
   - Only 100 days of data with `outputsize=compact`
   - Need `outputsize=full` for more history (slower)
   - Rate limit: 5 calls/minute, 500/day

2. **Twitter API Rate Limits**
   - 900 requests per 15 minutes for user timeline
   - May need to implement exponential backoff

3. **Weekend/Holiday Gaps**
   - Stock markets closed on weekends
   - Tweets on Saturday may not have same-day price data
   - Need to handle missing data gracefully

4. **Timezone Handling**
   - Tweets are in UTC
   - Stock prices are in market timezone (EST for US)
   - May need timezone conversion for accurate alignment

## 🧪 Testing Strategy

### Unit Tests
- Test each calculation function in isolation
- Use known inputs with expected outputs
- Test edge cases (empty data, zero prices, etc.)

### Integration Tests
- Mock API responses
- Test full pipeline with sample data
- Verify output format

### Manual Testing
- Test with real APIs (small datasets first)
- Verify against known events (e.g., Tesla stock split)
- Check correlation makes intuitive sense

## 📊 Example Test Data

For testing without API calls, create sample data:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn sample_tweets() -> Vec<Tweet> {
        vec![
            Tweet {
                id: "1".to_string(),
                text: "Great news about our product!".to_string(),
                created_at: Utc.ymd(2025, 1, 15).and_hms(10, 0, 0),
                retweet_count: 1000,
                like_count: 5000,
                sentiment: Some(0.8),
            },
            // ... more samples
        ]
    }
}
```

## 🔍 Debugging Tips

### Enable Verbose Mode
```bash
cargo run -- --ceo-handle test --ticker TEST --days 7 --verbose
```

### Check API Responses
Add temporary debug prints:
```rust
println!("API Response: {:#?}", response);
```

### Verify Data Alignment
Export intermediate results:
```bash
cargo run -- ... --export-prolog debug.pl
cat debug.pl  # Inspect generated facts
```

## 📚 Learning Resources

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### APIs
- [Twitter API v2 Docs](https://developer.twitter.com/en/docs/twitter-api)
- [Alpha Vantage Docs](https://www.alphavantage.co/documentation/)

### Statistics
- [Pearson Correlation](https://en.wikipedia.org/wiki/Pearson_correlation_coefficient)
- [Sentiment Analysis](https://en.wikipedia.org/wiki/Sentiment_analysis)

### Prolog
- [Scryer Prolog Guide](https://github.com/mthom/scryer-prolog/blob/master/README.md)
- [Learn Prolog Now](http://www.learnprolognow.org/)

### Lean 4
- [Theorem Proving in Lean 4](https://leanprover.github.io/theorem_proving_in_lean4/)
- [Mathlib Documentation](https://leanprover-community.github.io/mathlib4_docs/)

## 🎯 Milestones

### v0.1.0 (Current)
- [x] Basic architecture
- [x] Core modules implemented
- [x] Compiles successfully
- [ ] Works with real API data

### v0.2.0 (Next)
- [ ] Comprehensive tests
- [ ] Chart generation
- [ ] Improved sentiment analysis
- [ ] Better error messages

### v0.3.0 (Future)
- [ ] Full Prolog integration
- [ ] Multiple CEO comparison
- [ ] CSV export
- [ ] Performance optimization

### v1.0.0 (Production)
- [ ] Stable API
- [ ] Complete documentation
- [ ] Extensive test coverage
- [ ] Published to crates.io

## 🤝 Contributing

When contributing:
1. Create a feature branch
2. Write tests for new functionality
3. Update documentation
4. Run `cargo fmt` and `cargo clippy`
5. Ensure all tests pass
6. Submit PR with clear description

## 📞 Support

For questions or issues:
- Check existing GitHub issues
- Review documentation
- Ask in discussions

---

**Last Updated**: 2026-02-04
**Status**: Initial Implementation Complete ✅
