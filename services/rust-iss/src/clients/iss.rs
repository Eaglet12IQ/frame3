use super::{HttpClient, HttpClientConfig, IssClient, Result as ClientResult};
use async_trait::async_trait;
use serde_json::Value;

/// ISS Position API Client implementation
#[derive(Clone)]
pub struct IssClientImpl {
    http_client: HttpClient,
    base_url: String,
}

impl IssClientImpl {
    /// Create a new ISS client
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url: "https://api.wheretheiss.at".to_string(),
        }
    }

    /// Create ISS client with custom base URL (for testing)
    pub fn with_base_url(config: HttpClientConfig, base_url: String) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url,
        }
    }
}

#[async_trait]
impl IssClient for IssClientImpl {
    async fn fetch_iss_position(&self) -> ClientResult<Value> {
        let url = format!("{}/v1/satellites/25544", self.base_url);
        self.http_client.get_with_retry(&url, &[]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_iss_client_creation() {
        let config = HttpClientConfig::default();
        let client = IssClientImpl::new(config);
        assert_eq!(client.base_url, "https://api.wheretheiss.at");
    }

    #[tokio::test]
    async fn test_iss_client_with_custom_url() {
        let config = HttpClientConfig::default();
        let client = IssClientImpl::with_base_url(config, "https://test.iss.api".to_string());
        assert_eq!(client.base_url, "https://test.iss.api");
    }
}
