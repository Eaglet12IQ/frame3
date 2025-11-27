pub mod iss;
pub mod osdr;
pub mod cache;

pub use iss::*;
pub use osdr::*;
pub use cache::*;

use axum::http::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
    pub status: u16,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: status.as_u16(),
        }
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
}

impl From<ApiError> for (StatusCode, String) {
    fn from(err: ApiError) -> Self {
        (StatusCode::from_u16(err.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), err.message)
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
