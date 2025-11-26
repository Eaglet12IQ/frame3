use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use crate::config::HttpClientConfig;

/// Common client error type
#[derive(Debug, Clone)]
pub enum ClientError {
    HttpError(String),
    TimeoutError(String),
    ParseError(String),
    RateLimitError(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ClientError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            ClientError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ClientError::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

pub type Result<T> = std::result::Result<T, ClientError>;

/// Base HTTP client with common functionality
#[derive(Clone)]
pub struct HttpClient {
    client: std::sync::Arc<Client>,
    config: HttpClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client with configuration
    pub fn new(config: HttpClientConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client: std::sync::Arc::new(client), config }
    }

    /// Make a GET request with retry logic
    pub async fn get_with_retry(&self, url: &str, query_params: &[(&str, &str)]) -> Result<Value> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            match self.make_request(url, query_params).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < self.config.max_retries {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ClientError::HttpError("Max retries exceeded".to_string())))
    }

    /// Make a single HTTP request
    async fn make_request(&self, url: &str, query_params: &[(&str, &str)]) -> Result<Value> {
        let mut request = self.client.get(url);

        for (key, value) in query_params {
            request = request.query(&[(key, value)]);
        }

        let response = request.send().await
            .map_err(|e| ClientError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(ClientError::RateLimitError("Rate limit exceeded".to_string()));
            }
            return Err(ClientError::HttpError(format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))));
        }

        response.json().await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse JSON response: {}", e)))
    }
}

/// NASA API Client trait
#[async_trait]
pub trait NasaClient {
    async fn fetch_osdr_datasets(&self) -> Result<Value>;
    async fn fetch_apod(&self, api_key: Option<&str>) -> Result<Value>;
    async fn fetch_neo_feed(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> Result<Value>;
    async fn fetch_donki_flr(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> Result<Value>;
    async fn fetch_donki_cme(&self, start_date: &str, end_date: &str, api_key: Option<&str>) -> Result<Value>;
}

/// ISS Position API Client trait
#[async_trait]
pub trait IssClient {
    async fn fetch_iss_position(&self) -> Result<Value>;
}

/// SpaceX API Client trait
#[async_trait]
pub trait SpaceXClient {
    async fn fetch_next_launch(&self) -> Result<Value>;
    async fn fetch_latest_launch(&self) -> Result<Value>;
    async fn fetch_upcoming_launches(&self) -> Result<Value>;
}

// Re-export client implementations
pub mod nasa;
pub mod iss;
pub mod spacex;

pub use nasa::NasaClientImpl;
pub use iss::IssClientImpl;
pub use spacex::SpaceXClientImpl;
