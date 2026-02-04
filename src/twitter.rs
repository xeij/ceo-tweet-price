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
    _days: u32, // Days parameter is ignored in favor of strict count limit
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
    
    // Step 2: Fetch latest tweets (STRICT LIMIT: 50 tweets)
    // We strictly limit to 50 tweets to prevent excessive API credit usage
    // This provides enough data for correlation without breaking the bank
    let max_tweets = 50; 
    
    if verbose {
        println!("  → Fetching latest {} tweets...", max_tweets);
    }
    
    let tweets = fetch_user_tweets(&user_id, bearer_token, max_tweets, verbose).await?;
    
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

/// Fetch tweets for a user with a strict count limit
async fn fetch_user_tweets(
    user_id: &str,
    bearer_token: &str,
    max_tweets: usize,
    verbose: bool,
) -> Result<Vec<Tweet>> {
    let client = reqwest::Client::new();
    let mut all_tweets = Vec::new();
    let mut next_token: Option<String> = None;
    
    // Calculate how many to fetch in this batch (API max is 100)
    // We cap it at max_tweets or 100, whichever is smaller
    let fetch_count = std::cmp::min(max_tweets, 100);
    
    // Loop until we have enough tweets or run out of pages
    while all_tweets.len() < max_tweets {
        let mut url = format!(
            "{}/users/{}/tweets?max_results={}&tweet.fields=created_at,public_metrics",
            TWITTER_API_BASE,
            user_id,
            fetch_count
        );
        
        if let Some(token) = &next_token {
            url.push_str(&format!("&pagination_token={}", token));
        }
        
        // Exclude retweets and replies to ensure quality original content
        url.push_str("&exclude=retweets,replies");
        
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
                // If we reached our limit, stop processing
                if all_tweets.len() >= max_tweets {
                    break;
                }
                
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
            // If no more pages or we've reached our limit, break
            if next_token.is_none() || all_tweets.len() >= max_tweets {
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
