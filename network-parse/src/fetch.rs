use async_trait::async_trait;

use crate::error::{FeedError, FeedNotFoundError, FetchError};

#[async_trait]
pub trait FeedFetcher {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, FeedError>;
}

#[derive(Clone)]
pub struct RequestFetcher;

#[async_trait]
impl FeedFetcher for RequestFetcher {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, FeedError> {
        let resp = reqwest::get(url).await.map_err(|e| {
            FeedError::Fetch(FetchError {
                message: e.to_string(),
            })
        })?;
        if resp.status() == 404 {
            return Err(FeedError::NotFound(FeedNotFoundError {
                message: format!("404: {url}"),
            }));
        }
        let bytes = resp.bytes().await.map_err(|e| {
            FeedError::Fetch(FetchError {
                message: e.to_string(),
            })
        })?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Network test — hits live server, inherently flaky. Run explicitly:
    //   cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn fetch_real_rss() {
        let fetcher = RequestFetcher;
        let bytes = fetcher
            .fetch("https://hnrss.org/frontpage")
            .await
            .expect("fetch should succeed");
        assert!(!bytes.is_empty());
    }

    // Network test — depends on external server (httpbin) health. Run explicitly:
    //   cargo test -- --ignored
    // TODO: replace with wiremock for deterministic local mocking.
    #[tokio::test]
    #[ignore]
    async fn fetch_404_returns_not_found() {
        let fetcher = RequestFetcher;
        let result = fetcher.fetch("https://httpbin.org/status/404").await;
        assert!(matches!(result, Err(FeedError::NotFound(_))));
    }
}
