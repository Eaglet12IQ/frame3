use super::{HttpClient, HttpClientConfig, NasaClient, Result as ClientResult};
use async_trait::async_trait;
use serde_json::Value;

/// NASA API Client implementation
#[derive(Clone)]
pub struct NasaClientImpl {
    http_client: HttpClient,
    base_url: String,
}

impl NasaClientImpl {
    /// Create a new NASA client
    pub fn new(config: HttpClientConfig) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url: "https://api.nasa.gov".to_string(),
        }
    }

    /// Create NASA client with custom base URL (for testing)
    pub fn with_base_url(config: HttpClientConfig, base_url: String) -> Self {
        Self {
            http_client: HttpClient::new(config),
            base_url,
        }
    }
}

#[async_trait]
impl NasaClient for NasaClientImpl {
    async fn fetch_osdr_datasets(&self) -> ClientResult<Value> {
        let url = "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json";
        self.http_client.get_with_retry(url, &[]).await
    }

    async fn fetch_apod(&self, api_key: Option<&str>) -> ClientResult<Value> {
        let url = format!("{}/planetary/apod", self.base_url);
        let mut params = vec![("thumbs", "true")];

        if let Some(key) = api_key {
            params.push(("api_key", key));
        }

        self.http_client.get_with_retry(&url, &params).await
    }

    async fn fetch_neo_feed(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> ClientResult<Value> {
        let url = format!("{}/neo/rest/v1/feed", self.base_url);
        let mut params = vec![
            ("start_date", start_date),
            ("end_date", end_date),
        ];

        if let Some(key) = api_key {
            params.push(("api_key", key));
        }

        self.http_client.get_with_retry(&url, &params).await
    }

    async fn fetch_donki_flr(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> ClientResult<Value> {
        let url = format!("{}/DONKI/FLR", self.base_url);
        let mut params = vec![
            ("startDate", start_date),
            ("endDate", end_date),
        ];

        if let Some(key) = api_key {
            params.push(("api_key", key));
        }

        self.http_client.get_with_retry(&url, &params).await
    }

    async fn fetch_donki_cme(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> ClientResult<Value> {
        let url = format!("{}/DONKI/CME", self.base_url);
        let mut params = vec![
            ("startDate", start_date),
            ("endDate", end_date),
        ];

        if let Some(key) = api_key {
            params.push(("api_key", key));
        }

        self.http_client.get_with_retry(&url, &params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_nasa_client_creation() {
        let config = HttpClientConfig::default();
        let client = NasaClientImpl::new(config);
        assert_eq!(client.base_url, "https://api.nasa.gov");
    }

    #[tokio::test]
    async fn test_nasa_client_with_custom_url() {
        let config = HttpClientConfig::default();
        let client = NasaClientImpl::with_base_url(config, "https://test.nasa.gov".to_string());
        assert_eq!(client.base_url, "https://test.nasa.gov");
    }
}
