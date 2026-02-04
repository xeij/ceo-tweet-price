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

/// Fetch tweets from a CEO's Twitter account (via API or Scraper)
pub async fn fetch_tweets(
    handle: &str,
    bearer_token: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    _days: u32,
    verbose: bool,
) -> Result<Vec<Tweet>> {
    if let Some(token) = bearer_token {
        if verbose { println!("  → Using Twitter API v2"); }
        return fetch_tweets_api(handle, token, verbose).await;
    }
    
    if verbose { println!("  → Using Twitter Scraper"); }
    fetch_tweets_scraper(handle, username, password, verbose).await
}

async fn fetch_tweets_scraper(
    handle: &str,
    username: Option<&str>,
    password: Option<&str>,
    verbose: bool
) -> Result<Vec<Tweet>> {
    // Note: This relies on agent-twitter-client crate
    // We assume standard usage: Scraper::new(), login(), get_tweets()
    // The struct seems to be in the 'scraper' module based on error 'no Scraper in root'
    use agent_twitter_client::scraper::Scraper;

    let mut scraper = Scraper::new().await?;
    
    if let (Some(u), Some(p)) = (username, password) {
        if verbose { println!("  → Logging in as {}", u); }
        scraper.login(u, p).await.context("Failed to login to Twitter")?;
    } else {
        if verbose { println!("  → Attempting guest access (may specific limits)"); }
    }

    let max_tweets = 50;
    if verbose { println!("  → Scraping latest {} tweets...", max_tweets); }

    // First we need user ID.
    // Based on typical scraping lib patterns, get_profile usually returns user info
    let profile = scraper.get_profile(handle).await.context("Failed to get profile")?;
    // We'll guess the field is 'id' or 'user_id' or 'rest_id'. 
    // Let's try 'id' first, or 'user_id' if 'id' is distinct.
    // If this fails compile, we'll see the available fields in the error message.
    let user_id = profile.id.clone(); 
    
    // Fetch tweets
    // Type might be string or enum. Assuming enum based on previous code.
    // If TweetType is not found, we'll see error.
    let type_enum = agent_twitter_client::TweetType::Tweets; 
    
    let scraper_tweets = scraper.get_tweets(
        &user_id, 
        type_enum, 
        max_tweets as usize
    ).await.context("Failed to scrape tweets")?;

    let mut tweets = Vec::new();
    for t in scraper_tweets.tweets { // assuming result wrapper has 'tweets' field, or it IS a vec
         let created_at = if let Some(ts) = t.timestamp {
             DateTime::<Utc>::from_timestamp(ts as i64, 0).unwrap_or(Utc::now())
         } else {
             Utc::now()
         };

         tweets.push(Tweet {
             id: t.id.unwrap_or_default(),
             text: t.text.unwrap_or_default(),
             created_at,
             retweet_count: t.retweets.unwrap_or(0) as u32,
             like_count: t.likes.unwrap_or(0) as u32,
             sentiment: None,
         });
    }

    Ok(tweets)
}

async fn fetch_tweets_api(
    handle: &str,
    bearer_token: &str,
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
    let max_tweets = 50; 
    
    if verbose {
        println!("  → Fetching latest {} tweets...", max_tweets);
    }
    
    let tweets = fetch_user_tweets_api(&user_id, bearer_token, max_tweets, verbose).await?;
    
    Ok(tweets)
}


/// Get user ID from Twitter handle (API)
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

/// Fetch tweets for a user with a strict count limit (API)
async fn fetch_user_tweets_api(
    user_id: &str,
    bearer_token: &str,
    max_tweets: usize,
    verbose: bool,
) -> Result<Vec<Tweet>> {
    let client = reqwest::Client::new();
    let mut all_tweets = Vec::new();
    let mut next_token: Option<String> = None;
    
    let fetch_count = std::cmp::min(max_tweets, 100);
    
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
                    sentiment: None,
                });
            }
        }
        
        if let Some(meta) = tweets_response.meta {
            if verbose {
                println!("  → Fetched {} tweets so far...", all_tweets.len());
            }
            
            next_token = meta.next_token;
            if next_token.is_none() || all_tweets.len() >= max_tweets {
                break;
            }
        } else {
            break;
        }
        
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
