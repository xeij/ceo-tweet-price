//! Twitter API integration for fetching CEO tweets.
//!
//! This module handles authentication and fetching tweets from the Twitter API v2.
//! It uses reqwest for HTTP requests and handles rate limiting gracefully.

use crate::models::Tweet;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

/// Twitter API v2 base URL
const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

/// Response from Twitter API user lookup
#[derive(Debug, Deserialize)]
struct UserLookupResponse {
    data: UserData,
}

#[derive(Debug, Deserialize)]
struct UserData {
    id: String,
    username: String,
}

/// Response from Twitter API tweets endpoint
#[derive(Debug, Deserialize)]
struct TweetsResponse {
    data: Option<Vec<TweetData>>,
    meta: Option<Meta>,
}

#[derive(Debug, Deserialize)]
struct TweetData {
    id: String,
    text: String,
    created_at: String,
    public_metrics: Option<PublicMetrics>,
}

#[derive(Debug, Deserialize)]
struct PublicMetrics {
    retweet_count: u32,
    like_count: u32,
}

#[derive(Debug, Deserialize)]
struct Meta {
    result_count: usize,
    next_token: Option<String>,
}

/// Fetch tweets from a CEO's Twitter account
///
/// # Arguments
/// * `handle` - Twitter handle without @ symbol
/// * `bearer_token` - Twitter API Bearer Token
/// * `days` - Number of days to look back
/// * `verbose` - Enable verbose logging
///
/// # Returns
/// Vector of tweets ordered by creation date (newest first)
pub async fn fetch_tweets(
    handle: &str,
    bearer_token: &str,
    days: u32,
    verbose: bool,
) -> Result<Vec<Tweet>> {
    if verbose {
        println!("  → Looking up user ID for @{}", handle);
    }
    
    // Step 1: Get user ID from handle
    let user_id = get_user_id(handle, bearer_token).await?;
    
    if verbose {
        println!("  → User ID: {}", user_id);
    }
    
    // Step 2: Calculate date range
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(days as i64);
    
    if verbose {
        println!("  → Fetching tweets from {} to {}", 
                 start_date.format("%Y-%m-%d"),
                 end_date.format("%Y-%m-%d"));
    }
    
    // Step 3: Fetch tweets with pagination
    let tweets = fetch_user_tweets(&user_id, bearer_token, start_date, end_date, verbose).await?;
    
    Ok(tweets)
}

/// Get user ID from Twitter handle
async fn get_user_id(handle: &str, bearer_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/users/by/username/{}", TWITTER_API_BASE, handle);
    
    let response = client
        .get(&url)
        .bearer_auth(bearer_token)
        .send()
        .await
        .context("Failed to fetch user data from Twitter API")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Twitter API error ({}): {}", status, body);
    }
    
    let user_response: UserLookupResponse = response
        .json()
        .await
        .context("Failed to parse user lookup response")?;
    
    Ok(user_response.data.id)
}

/// Fetch tweets for a user within a date range
async fn fetch_user_tweets(
    user_id: &str,
    bearer_token: &str,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    verbose: bool,
) -> Result<Vec<Tweet>> {
    let client = reqwest::Client::new();
    let mut all_tweets = Vec::new();
    let mut next_token: Option<String> = None;
    
    // Twitter API pagination loop
    loop {
        let mut url = format!(
            "{}/users/{}/tweets?max_results=100&tweet.fields=created_at,public_metrics&start_time={}&end_time={}",
            TWITTER_API_BASE,
            user_id,
            start_date.format("%Y-%m-%dT%H:%M:%SZ"),
            end_date.format("%Y-%m-%dT%H:%M:%SZ")
        );
        
        if let Some(token) = &next_token {
            url.push_str(&format!("&pagination_token={}", token));
        }
        
        let response = client
            .get(&url)
            .bearer_auth(bearer_token)
            .send()
            .await
            .context("Failed to fetch tweets from Twitter API")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Twitter API error ({}): {}", status, body);
        }
        
        let tweets_response: TweetsResponse = response
            .json()
            .await
            .context("Failed to parse tweets response")?;
        
        if let Some(data) = tweets_response.data {
            for tweet_data in data {
                let created_at = DateTime::parse_from_rfc3339(&tweet_data.created_at)
                    .context("Failed to parse tweet timestamp")?
                    .with_timezone(&Utc);
                
                let metrics = tweet_data.public_metrics.unwrap_or(PublicMetrics {
                    retweet_count: 0,
                    like_count: 0,
                });
                
                all_tweets.push(Tweet {
                    id: tweet_data.id,
                    text: tweet_data.text,
                    created_at,
                    retweet_count: metrics.retweet_count,
                    like_count: metrics.like_count,
                    sentiment: None, // Will be calculated later
                });
            }
        }
        
        // Check for more pages
        if let Some(meta) = tweets_response.meta {
            if verbose {
                println!("  → Fetched {} tweets so far...", all_tweets.len());
            }
            
            next_token = meta.next_token;
            if next_token.is_none() {
                break;
            }
        } else {
            break;
        }
        
        // Rate limiting: sleep briefly between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    Ok(all_tweets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitter_api_base_url() {
        assert_eq!(TWITTER_API_BASE, "https://api.twitter.com/2");
    }
}
