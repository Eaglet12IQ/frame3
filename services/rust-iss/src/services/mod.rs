mod iss;
mod osdr;
mod cache;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::result;

use crate::domain::*;
use crate::repo::*;

/// Common service error type
#[derive(Debug, Clone)]
pub enum ServiceError {
    RepositoryError(String),
    ValidationError(String),
    ExternalApiError(String),
    BusinessLogicError(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::ExternalApiError(msg) => write!(f, "External API error: {}", msg),
            ServiceError::BusinessLogicError(msg) => write!(f, "Business logic error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

pub type Result<T> = result::Result<T, ServiceError>;

/// ISS Service trait
#[async_trait]
pub trait IssService {
    async fn fetch_and_store_iss_data(&self, url: &str) -> Result<IssData>;
    async fn get_latest_iss_data(&self) -> Result<Option<IssData>>;
    async fn get_iss_trend_analysis(&self) -> Result<IssTrend>;
    async fn trigger_iss_fetch(&self) -> Result<IssData>;
}

/// OSDR Service trait
#[async_trait]
pub trait OsdrService {
    async fn sync_osdr_data(&self, api_url: &str) -> Result<usize>;
    async fn get_osdr_items(&self, limit: i64) -> Result<Vec<OsdrItem>>;
    async fn get_osdr_item_count(&self) -> Result<i64>;
}

/// Cache Service trait
#[async_trait]
pub trait CacheService {
    async fn fetch_and_cache_apod(&self, api_key: Option<&str>) -> Result<SpaceCache>;
    async fn fetch_and_cache_neo_feed(&self, api_key: Option<&str>) -> Result<SpaceCache>;
    async fn fetch_and_cache_donki_data(&self, api_key: Option<&str>) -> Result<Vec<SpaceCache>>;
    async fn fetch_and_cache_spacex_next(&self) -> Result<SpaceCache>;
    async fn get_latest_cache_entry(&self, source: &str) -> Result<Option<SpaceCache>>;
    async fn refresh_multiple_sources(&self, sources: Vec<String>, api_key: Option<&str>) -> Result<Vec<String>>;
    async fn get_space_summary(&self) -> Result<SpaceSummary>;
    async fn store_space_cache(&self, source: String, payload: serde_json::Value) -> Result<()>;
}

/// ISS Trend analysis result
#[derive(Debug, Clone, serde::Serialize)]
pub struct IssTrend {
    pub movement: bool,
    pub delta_km: f64,
    pub dt_sec: f64,
    pub velocity_kmh: Option<f64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub from_lat: Option<f64>,
    pub from_lon: Option<f64>,
    pub to_lat: Option<f64>,
    pub to_lon: Option<f64>,
}

/// Space data summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct SpaceSummary {
    pub apod: Option<SpaceCache>,
    pub neo: Option<SpaceCache>,
    pub flr: Option<SpaceCache>,
    pub cme: Option<SpaceCache>,
    pub spacex: Option<SpaceCache>,
    pub iss: Option<IssData>,
    pub osdr_count: i64,
}

// Re-export service implementations
pub use crate::services::iss::IssServiceImpl;
pub use crate::services::osdr::OsdrServiceImpl;
pub use crate::services::cache::CacheServiceImpl;
