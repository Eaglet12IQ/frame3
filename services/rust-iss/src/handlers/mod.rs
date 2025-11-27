pub mod iss;
pub mod osdr;
pub mod cache;

pub use iss::*;
pub use osdr::*;
pub use cache::*;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::{error, warn};

use crate::domain::DomainError;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: status.as_u16(),
            trace_id: None,
        }
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Log the error based on status code
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                error!("Internal server error: {}", self.message);
            }
            StatusCode::BAD_REQUEST => {
                warn!("Bad request: {}", self.message);
            }
            _ => {
                warn!("API error ({}): {}", status, self.message);
            }
        }

        (status, Json(self)).into_response()
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::ValidationError(msg) => ApiError::bad_request(msg),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::internal_error(err.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        error!("Database error: {:?}", err);
        ApiError::internal_error("Database operation failed")
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        error!("HTTP client error: {:?}", err);
        ApiError::service_unavailable("External service unavailable")
    }
}

use crate::AppState;
use crate::services::{IssService, OsdrService, CacheService};

pub async fn fetch_and_store_iss(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.iss_service.trigger_iss_fetch().await?;
    Ok(())
}

pub async fn fetch_and_store_osdr(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.osdr_service.sync_osdr_data(&st.config.osdr.api_url).await?;
    Ok(())
}

pub async fn fetch_apod(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_and_cache_apod(st.config.nasa.api_key.as_deref()).await?;
    Ok(())
}

pub async fn fetch_neo_feed(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_and_cache_neo_feed(st.config.nasa.api_key.as_deref()).await?;
    Ok(())
}

pub async fn fetch_donki_flr(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_donki_flr(st.config.nasa.api_key.as_deref()).await?;
    Ok(())
}

pub async fn fetch_donki_cme(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_donki_cme(st.config.nasa.api_key.as_deref()).await?;
    Ok(())
}

pub async fn fetch_donki(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_and_cache_donki_data(st.config.nasa.api_key.as_deref()).await?;
    Ok(())
}

pub async fn fetch_spacex_next(st: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    st.cache_service.fetch_and_cache_spacex_next().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new(StatusCode::BAD_REQUEST, "Test error");
        assert_eq!(error.status, 400);
        assert_eq!(error.message, "Test error");
        assert!(error.trace_id.is_none());
    }

    #[test]
    fn test_api_error_with_trace_id() {
        let error = ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Server error")
            .with_trace_id("trace-123");
        assert_eq!(error.status, 500);
        assert_eq!(error.message, "Server error");
        assert_eq!(error.trace_id, Some("trace-123".to_string()));
    }

    #[test]
    fn test_api_error_convenience_methods() {
        let not_found = ApiError::not_found("Not found");
        assert_eq!(not_found.status, 404);
        assert_eq!(not_found.message, "Not found");

        let bad_request = ApiError::bad_request("Bad request");
        assert_eq!(bad_request.status, 400);
        assert_eq!(bad_request.message, "Bad request");

        let unavailable = ApiError::service_unavailable("Service unavailable");
        assert_eq!(unavailable.status, 503);
        assert_eq!(unavailable.message, "Service unavailable");
    }
}
