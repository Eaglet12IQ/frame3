use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use std::time::Duration;

use crate::domain::*;
use crate::repo::{RepoError, IssRepo, OsdrRepo, CacheRepo};
use crate::services::*;

/// Implementation of Cache Service
#[derive(Clone)]
pub struct CacheServiceImpl<R: CacheRepo + IssRepo + OsdrRepo + Sync + Clone> {
    repo: R,
}

impl<R: CacheRepo + IssRepo + OsdrRepo + Sync + Clone> CacheServiceImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl<R: CacheRepo + IssRepo + OsdrRepo + Sync + Clone> CacheService for CacheServiceImpl<R> {
    async fn fetch_and_cache_apod(&self, api_key: Option<&str>) -> Result<SpaceCache> {
        let url = "https://api.nasa.gov/planetary/apod";
        let client = create_http_client()?;

        let mut request = client.get(url).query(&[("thumbs", "true")]);
        if let Some(key) = api_key {
            request = request.query(&[("api_key", key)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("APOD API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(format!(
                "APOD API returned status: {}", response.status()
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse APOD response: {}", e)))?;

        let cache_entry = SpaceCache::new("apod".to_string(), json);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(cache_entry)
    }

    async fn fetch_and_cache_neo_feed(&self, api_key: Option<&str>) -> Result<SpaceCache> {
        let today = Utc::now().date_naive();
        let start_date = today - chrono::Days::new(2);
        let url = "https://api.nasa.gov/neo/rest/v1/feed";
        let client = create_http_client()?;

        let mut request = client.get(url).query(&[
            ("start_date", start_date.to_string()),
            ("end_date", today.to_string()),
        ]);

        if let Some(key) = api_key {
            request = request.query(&[("api_key", key)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("NEO API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(format!(
                "NEO API returned status: {}", response.status()
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse NEO response: {}", e)))?;

        let cache_entry = SpaceCache::new("neo".to_string(), json);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(cache_entry)
    }

    async fn fetch_and_cache_donki_data(&self, api_key: Option<&str>) -> Result<Vec<SpaceCache>> {
        let mut results = Vec::new();

        // Fetch FLR data
        if let Ok(flr) = self.fetch_donki_flr(api_key).await {
            results.push(flr);
        }

        // Fetch CME data
        if let Ok(cme) = self.fetch_donki_cme(api_key).await {
            results.push(cme);
        }

        Ok(results)
    }

    async fn fetch_and_cache_spacex_next(&self) -> Result<SpaceCache> {
        let url = "https://api.spacexdata.com/v4/launches/next";
        let client = create_http_client()?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("SpaceX API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ServiceError::ExternalApiError(format!(
                "SpaceX API returned status: {}", response.status()
            )));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse SpaceX response: {}", e)))?;

        let cache_entry = SpaceCache::new("spacex".to_string(), json);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(cache_entry)
    }

    async fn get_latest_cache_entry(&self, source: &str) -> Result<Option<SpaceCache>> {
        self.repo
            .get_latest_cache_entry(source)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))
    }

    async fn refresh_multiple_sources(&self, sources: Vec<String>, api_key: Option<&str>) -> Result<Vec<String>> {
        let mut refreshed = Vec::new();

        for source in sources {
            let result = match source.to_lowercase().as_str() {
                "apod" => {
                    let _ = self.fetch_and_cache_apod(api_key).await;
                    Some("apod".to_string())
                }
                "neo" => {
                    let _ = self.fetch_and_cache_neo_feed(api_key).await;
                    Some("neo".to_string())
                }
                "flr" => {
                    let _ = self.fetch_donki_flr(api_key).await;
                    Some("flr".to_string())
                }
                "cme" => {
                    let _ = self.fetch_donki_cme(api_key).await;
                    Some("cme".to_string())
                }
                "spacex" => {
                    let _ = self.fetch_and_cache_spacex_next().await;
                    Some("spacex".to_string())
                }
                _ => None,
            };

            if let Some(refreshed_source) = result {
                refreshed.push(refreshed_source);
            }
        }

        Ok(refreshed)
    }

    async fn get_space_summary(&self) -> Result<SpaceSummary> {
        let apod = self.get_latest_cache_entry("apod").await?;
        let neo = self.get_latest_cache_entry("neo").await?;
        let flr = self.get_latest_cache_entry("flr").await?;
        let cme = self.get_latest_cache_entry("cme").await?;
        let spacex = self.get_latest_cache_entry("spacex").await?;
        let iss = self.repo
            .get_latest_iss_data()
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;
        let osdr_count = self.repo
            .count_osdr_items()
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(SpaceSummary {
            apod,
            neo,
            flr,
            cme,
            spacex,
            iss,
            osdr_count,
        })
    }

    async fn store_space_cache(&self, source: String, payload: serde_json::Value) -> Result<()> {
        let cache_entry = SpaceCache::new(source, payload);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}

impl<R: CacheRepo + IssRepo + OsdrRepo + Sync + Clone> CacheServiceImpl<R> {
    /// Fetch DONKI FLR data
    pub async fn fetch_donki_flr(&self, api_key: Option<&str>) -> Result<SpaceCache> {
        let (from, to) = get_last_days_range(5);
        let url = "https://api.nasa.gov/DONKI/FLR";
        let client = create_http_client()?;

        let mut request = client.get(url).query(&[("startDate", from), ("endDate", to)]);
        if let Some(key) = api_key {
            request = request.query(&[("api_key", key)]);
        }

        let json: Value = request
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("DONKI FLR request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse DONKI FLR response: {}", e)))?;

        let cache_entry = SpaceCache::new("flr".to_string(), json);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(cache_entry)
    }

    /// Fetch DONKI CME data
    pub async fn fetch_donki_cme(&self, api_key: Option<&str>) -> Result<SpaceCache> {
        let (from, to) = get_last_days_range(5);
        let url = "https://api.nasa.gov/DONKI/CME";
        let client = create_http_client()?;

        let mut request = client.get(url).query(&[("startDate", from), ("endDate", to)]);
        if let Some(key) = api_key {
            request = request.query(&[("api_key", key)]);
        }

        let json: Value = request
            .send()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("DONKI CME request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to parse DONKI CME response: {}", e)))?;

        let cache_entry = SpaceCache::new("cme".to_string(), json);
        cache_entry
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        self.repo
            .insert_cache_entry(&cache_entry)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(cache_entry)
    }
}

/// Create HTTP client with default timeout
fn create_http_client() -> crate::services::Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| ServiceError::ExternalApiError(format!("Failed to create HTTP client: {}", e)))
}

/// Get date range for last N days
fn get_last_days_range(days: i64) -> (String, String) {
    let to = Utc::now().date_naive();
    let from = to - chrono::Days::new(days as u64);
    (from.to_string(), to.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::RepoError;

    // Mock repository for testing
    struct MockCacheRepo;

    #[async_trait]
    impl CacheRepo for MockCacheRepo {
        async fn insert_cache_entry(&self, _entry: &SpaceCache) -> crate::repo::Result<i64> {
            Ok(1)
        }

        async fn get_latest_cache_entry(&self, _source: &str) -> crate::repo::Result<Option<SpaceCache>> {
            Ok(None)
        }

        async fn get_cache_entries(&self, _source: &str, _limit: i64) -> crate::repo::Result<Vec<SpaceCache>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl IssRepo for MockCacheRepo {
        async fn insert_iss_data(&self, _data: &IssData) -> crate::repo::Result<i64> {
            Ok(1)
        }

        async fn get_latest_iss_data(&self) -> crate::repo::Result<Option<IssData>> {
            Ok(None)
        }

        async fn get_iss_data_range(&self, _limit: i64) -> crate::repo::Result<Vec<IssData>> {
            Ok(vec![])
        }

        async fn get_iss_trend_data(&self) -> crate::repo::Result<Vec<IssData>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl OsdrRepo for MockCacheRepo {
        async fn insert_or_update_osdr_item(&self, _item: &OsdrItem) -> crate::repo::Result<i64> {
            Ok(1)
        }

        async fn get_osdr_items(&self, _limit: i64) -> crate::repo::Result<Vec<OsdrItem>> {
            Ok(vec![])
        }

        async fn get_osdr_item_by_id(&self, _dataset_id: &str) -> crate::repo::Result<Option<OsdrItem>> {
            Ok(None)
        }

        async fn count_osdr_items(&self) -> crate::repo::Result<i64> {
            Ok(0)
        }
    }

    #[test]
    fn test_get_last_days_range() {
        let (from, to) = get_last_days_range(5);
        assert!(!from.is_empty());
        assert!(!to.is_empty());
        assert!(from < to);
    }

    #[tokio::test]
    async fn test_get_space_summary() {
        let service = CacheServiceImpl::new(MockCacheRepo);
        let result = service.get_space_summary().await;
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.osdr_count, 0);
        assert!(summary.apod.is_none());
    }
}
