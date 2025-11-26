use super::{HttpClient, HttpClientConfig, SpaceXClient, Result as ClientResult};
use async_trait::async_trait;
use serde_json::Value;

/// SpaceX API Client implementation
#[derive(Clone)]
pub struct SpaceXClientImpl {
    http_client: HttpClient,
    base_url: String,
}

impl SpaceXClientImpl {
    /// Create a new SpaceX client
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url: "https://api.spacexdata.com".to_string(),
        }
    }

    /// Create SpaceX client with custom base URL (for testing)
    pub fn with_base_url(config: HttpClientConfig, base_url: String) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url,
        }
    }
}

#[async_trait]
impl SpaceXClient for SpaceXClientImpl {
    async fn fetch_next_launch(&self) -> ClientResult<Value> {
        let url = format!("{}/v4/launches/next", self.base_url);
        self.http_client.get_with_retry(&url, &[]).await
    }

    async fn fetch_latest_launch(&self) -> ClientResult<Value> {
        let url = format!("{}/v4/launches/latest", self.base_url);
        self.http_client.get_with_retry(&url, &[]).await
    }

    async fn fetch_upcoming_launches(&self) -> ClientResult<Value> {
        let url = format!("{}/v4/launches/upcoming", self.base_url);
        self.http_client.get_with_retry(&url, &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_spacex_client_creation() {
        let config = HttpClientConfig::default();
        let client = SpaceXClientImpl::new(config);
        assert_eq!(client.base_url, "https://api.spacexdata.com");
    }

    #[tokio::test]
    async fn test_spacex_client_with_custom_url() {
        let config = HttpClientConfig::default();
        let client = SpaceXClientImpl::with_base_url(config, "https://test.spacex.api".to_string());
        assert_eq!(client.base_url, "https://test.spacex.api");
    }
}
