# Quick Start Guide

## 🚀 Get Up and Running in 5 Minutes

### Step 1: Build the Project
```bash
cd c:\Users\xeij\Desktop\ceo-tweet
cargo clean
cargo build --release
```

**Note**: If build fails with file locking errors, close all terminals/IDEs and try again.

### Step 2: Get API Keys

#### Twitter API Bearer Token
1. Go to https://developer.twitter.com/en/portal/dashboard
2. Create a new app (or use existing)
3. Navigate to "Keys and tokens"
4. Generate/copy your "Bearer Token"

#### Alpha Vantage API Key
1. Go to https://www.alphavantage.co/support/#api-key
2. Enter your email
3. Copy the API key from the confirmation page

### Step 3: Set Environment Variables

**PowerShell (Windows)**:
```powershell
$env:TWITTER_BEARER_TOKEN="your_twitter_bearer_token_here"
$env:STOCK_API_KEY="your_alpha_vantage_key_here"
```

**Or use CLI flags**:
```bash
--api-key-twitter YOUR_TOKEN --api-key-stocks YOUR_KEY
```

### Step 4: Run Your First Analysis

```bash
cargo run --release -- \
  --ceo-handle elonmusk \
  --ticker TSLA \
  --days 30 \
  --verbose
```

**Expected Output**:
```
🚀 CEO Tweet Analyzer Starting...

📱 Fetching tweets from @elonmusk...
  → Looking up user ID for @elonmusk
  → User ID: 44196397
  → Fetching tweets from 2026-01-05 to 2026-02-04
✅ Fetched 127 tweets

📊 Fetching stock prices for TSLA...
  → Fetching daily prices for TSLA
  → Requesting data from Alpha Vantage...
  → Parsing 30 daily records...
  → Returning 30 price points
✅ Fetched 30 price points

🔬 Analyzing tweet impacts and correlations...
  → Calculating sentiment for 127 tweets...
  → Aligning tweets with price data...
  → Calculating correlations...
✅ Analysis complete

🧠 Applying Prolog rules for pattern detection...
✅ Prolog analysis complete

📋 Results:

═══════════════════════════════════════════════════════════════════════════
  CEO Tweet Impact Analysis
═══════════════════════════════════════════════════════════════════════════
  CEO: @elonmusk
  Ticker: TSLA
  Period: 2026-01-05 to 2026-02-04
  Total Tweets: 127
  Tweets with Price Data: 89
═══════════════════════════════════════════════════════════════════════════

📊 Summary Statistics:
  Correlation (sentiment vs 1d change): 0.2341
  Correlation (sentiment vs 3d change): 0.1876
  Positive tweets → >3% rise (1d): 34.2%
  Positive tweets → >3% rise (3d): 41.7%

🎯 Most Impactful Tweets (by Prolog rules):
  [Top 5 tweets with sentiment and price changes]

✨ Analysis complete!
```

## 📊 More Examples

### Export Prolog Facts
```bash
cargo run --release -- \
  --ceo-handle elonmusk \
  --ticker TSLA \
  --days 90 \
  --export-prolog elon_tsla.pl
```

Then query with Scryer Prolog:
```bash
scryer-prolog elon_tsla.pl
?- impactful_tweet(X).
```

### JSON Output
```bash
cargo run --release -- \
  --ceo-handle elonmusk \
  --ticker TSLA \
  --days 60 \
  --output-format json > results.json
```

### Different CEO/Stock Pairs
```bash
# Tim Cook / Apple
cargo run --release -- --ceo-handle tim_cook --ticker AAPL --days 90

# Satya Nadella / Microsoft  
cargo run --release -- --ceo-handle satyanadella --ticker MSFT --days 180

# Jensen Huang / NVIDIA
cargo run --release -- --ceo-handle nvidiaai --ticker NVDA --days 120
```

## 🐛 Troubleshooting

### "Twitter API error (401)"
- Your Bearer Token is invalid or expired
- Regenerate token in Twitter Developer Portal
- Make sure you're using Bearer Token, not API Key

### "Alpha Vantage rate limit exceeded"
- Free tier: 5 calls/minute, 500/day
- Wait 60 seconds and try again
- Or get a premium key

### "No tweets found"
- Check the handle is correct (without @)
- Verify account has public tweets
- Try reducing `--days` parameter

### Build fails with "file in use" error
- Close all terminals and IDEs
- Run `cargo clean`
- Try again

## 📈 Understanding Results

### Correlation Values
- **-1.0 to -0.5**: Strong negative correlation (negative tweets → price drop)
- **-0.5 to -0.3**: Moderate negative correlation
- **-0.3 to +0.3**: Weak/no correlation
- **+0.3 to +0.5**: Moderate positive correlation
- **+0.5 to +1.0**: Strong positive correlation (positive tweets → price rise)

### Sentiment Scores
- **-1.0**: Very negative
- **-0.5**: Moderately negative
- **0.0**: Neutral
- **+0.5**: Moderately positive
- **+1.0**: Very positive

### Impactful Tweets
A tweet is marked "impactful" if:
- Sentiment magnitude > 0.3 (strong opinion)
- AND price change > 3% within 1-3 days

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_sentiment_positive
```

## 📚 Next Steps

1. **Read the full README.md** for detailed documentation
2. **Check DEVELOPMENT.md** for contributing guidelines
3. **Explore Lean proofs** in `lean/` directory
4. **Customize sentiment keywords** in `src/analysis.rs`
5. **Add new Prolog rules** in `src/prolog.rs`

## 💡 Tips

- Start with small `--days` values (30-90) to avoid rate limits
- Use `--verbose` to see what's happening
- Export Prolog facts to understand the data better
- Compare different time periods to see trends
- Try multiple CEO/stock pairs to find interesting correlations

## 🎯 Goals for Your Analysis

- Find correlation between CEO communication and stock performance
- Identify which types of tweets have the most impact
- Discover patterns in successful vs. unsuccessful announcements
- Build intuition about market sentiment

---

**Happy Analyzing! 🚀📊**

For questions, check the documentation or review the code - it's well-commented!
