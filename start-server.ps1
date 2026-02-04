# CEO Tweet Analyzer - Web Server Startup Script
# 
# This script starts the web dashboard server
# Make sure to set your API keys first!

Write-Host "CEO Tweet Analyzer - Web Dashboard" -ForegroundColor Cyan
Write-Host "======================================`n" -ForegroundColor Cyan

# Automatically load keys from setup-env.ps1 if it exists
if (Test-Path ".\setup-env.ps1") {
    Write-Host "Loading API keys from setup-env.ps1..." -ForegroundColor Cyan
    . .\setup-env.ps1
}


# Check if API keys are set
if (-not $env:TWITTER_BEARER_TOKEN) {
    Write-Host "ERROR: TWITTER_BEARER_TOKEN not set" -ForegroundColor Red
    Write-Host "`nPlease set your Twitter API Bearer Token:" -ForegroundColor Yellow
    Write-Host '  $env:TWITTER_BEARER_TOKEN="your_token_here"' -ForegroundColor White
    Write-Host "`nGet your token from: https://developer.twitter.com/en/portal/dashboard`n" -ForegroundColor Gray
    exit 1
}

if (-not $env:STOCK_API_KEY) {
    Write-Host "ERROR: STOCK_API_KEY not set" -ForegroundColor Red
    Write-Host "`nPlease set your Alpha Vantage API key:" -ForegroundColor Yellow
    Write-Host '  $env:STOCK_API_KEY="your_key_here"' -ForegroundColor White
    Write-Host "`nGet your key from: https://www.alphavantage.co/support/#api-key`n" -ForegroundColor Gray
    exit 1
}

Write-Host "API keys configured" -ForegroundColor Green
Write-Host "`nBuilding web server..." -ForegroundColor Cyan

# Build the web server
cargo build --bin web-server --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    Write-Host "Try running: cargo clean" -ForegroundColor Yellow
    exit 1
}

Write-Host "Build complete`n" -ForegroundColor Green

# Start the server
Write-Host "Starting web server..." -ForegroundColor Cyan
Write-Host "Dashboard will be available at: http://localhost:3000" -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop the server`n" -ForegroundColor Gray

cargo run --bin web-server --release

