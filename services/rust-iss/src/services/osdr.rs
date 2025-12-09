use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use std::time::Duration;

use crate::domain::*;
use crate::repo::*;
use crate::services::*;
use crate::clients::NasaClient;

/// Implementation of OSDR Service
#[derive(Clone)]
pub struct OsdrServiceImpl<R: OsdrRepo + Clone, C: NasaClient + Clone> {
    repo: R,
    client: C,
}

impl<R: OsdrRepo + Clone, C: NasaClient + Clone> OsdrServiceImpl<R, C> {
    pub fn new(repo: R, client: C) -> Self {
        Self { repo, client }
    }
}

#[async_trait]
impl<R: OsdrRepo + Clone + Sync, C: NasaClient + Clone + Sync> OsdrService for OsdrServiceImpl<R, C> {
    async fn sync_osdr_data(&self, _api_url: &str) -> crate::services::Result<usize> {
        // Fetch data from OSDR API using NasaClient
        let json: Value = self.client
            .fetch_osdr_datasets()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("Failed to fetch OSDR data: {}", e)))?;

        // Parse the response - handle different possible formats
        let items = parse_osdr_response(json)?;

        let mut written = 0usize;
        for item in items {
            // Create domain model and validate
            let osdr_item = extract_osdr_item_fields(&item);
            osdr_item
                .validate()
                .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

            // Store in repository
            self.repo
                .insert_or_update_osdr_item(&osdr_item)
                .await
                .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

            written += 1;
        }

        Ok(written)
    }

    async fn get_osdr_items(&self, limit: i64) -> crate::services::Result<Vec<OsdrItem>> {
        self.repo
            .get_osdr_items(limit)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))
    }

    async fn get_osdr_item_count(&self) -> crate::services::Result<i64> {
        self.repo
            .count_osdr_items()
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))
    }
}

/// Parse OSDR API response into a vector of JSON values
fn parse_osdr_response(json: Value) -> crate::services::Result<Vec<Value>> {
    if let Some(array) = json.as_array() {
        Ok(array.clone())
    } else if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        Ok(items.clone())
    } else if let Some(results) = json.get("results").and_then(|v| v.as_array()) {
        Ok(results.clone())
    } else {
        // Single object response
        Ok(vec![json])
    }
}

/// Extract OSDR item fields from raw JSON
fn extract_osdr_item_fields(raw: &Value) -> OsdrItem {
    let dataset_id = extract_string_field(raw, &["dataset_id", "id", "uuid", "studyId", "accession", "osdr_id"]);
    let title = extract_string_field(raw, &["title", "name", "label"]);
    let status = extract_string_field(raw, &["status", "state", "lifecycle"]);
    let updated_at = extract_timestamp_field(raw, &["updated", "updated_at", "modified", "lastUpdated", "timestamp"]);

    OsdrItem::with_fields(dataset_id, title, status, updated_at, raw.clone())
}

/// Extract string field from JSON using multiple possible keys
fn extract_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(field_value) = value.get(key) {
            if let Some(s) = field_value.as_str() {
                if !s.trim().is_empty() {
                    return Some(s.to_string());
                }
            } else if field_value.is_number() {
                return Some(field_value.to_string());
            }
        }
    }
    None
}

/// Extract timestamp field from JSON using multiple possible keys
fn extract_timestamp_field(value: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    for key in keys {
        if let Some(field_value) = value.get(key) {
            if let Some(s) = field_value.as_str() {
                // Try parsing as RFC3339 datetime
                if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                    return Some(dt);
                }
                // Try parsing as naive datetime (assuming UTC)
                if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return Some(Utc.from_utc_datetime(&ndt));
                }
            } else if let Some(n) = field_value.as_i64() {
                // Assume Unix timestamp
                return Some(Utc.timestamp_opt(n, 0).single().unwrap_or_else(Utc::now));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::RepoError;

    // Mock repository for testing
    #[derive(Clone)]
    struct MockOsdrRepo;

    #[async_trait]
    impl OsdrRepo for MockOsdrRepo {
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
    fn test_parse_osdr_response_array() {
        let json = serde_json::json!([
            {"dataset_id": "1", "title": "Test 1"},
            {"dataset_id": "2", "title": "Test 2"}
        ]);
        let result = parse_osdr_response(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_osdr_response_single() {
        let json = serde_json::json!({"dataset_id": "1", "title": "Test"});
        let result = parse_osdr_response(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_extract_osdr_item_fields() {
        let raw = serde_json::json!({
            "dataset_id": "123",
            "title": "Test Dataset",
            "status": "active",
            "updated": "2023-01-01T00:00:00Z"
        });
        let item = extract_osdr_item_fields(&raw);
        assert_eq!(item.dataset_id, Some("123".to_string()));
        assert_eq!(item.title, Some("Test Dataset".to_string()));
        assert_eq!(item.status, Some("active".to_string()));
        assert!(item.updated_at.is_some());
    }

    // Mock client for testing
    #[derive(Clone)]
    struct MockNasaClient;

    #[async_trait]
    impl NasaClient for MockNasaClient {
        async fn fetch_osdr_datasets(&self) -> ClientResult<Value> {
            Ok(serde_json::json!([{"dataset_id": "1", "title": "Test"}]))
        }

        async fn fetch_apod(&self, _api_key: Option<&str>) -> ClientResult<Value> {
            Ok(serde_json::json!({"title": "Test APOD"}))
        }

        async fn fetch_neo_feed(&self, _start_date: &str, _end_date: &str, _api_key: Option<&str>) -> ClientResult<Value> {
            Ok(serde_json::json!({"near_earth_objects": {}}))
        }

        async fn fetch_donki_flr(&self, _start_date: &str, _end_date: &str, _api_key: Option<&str>) -> ClientResult<Value> {
            Ok(serde_json::json!([]))
        }

        async fn fetch_donki_cme(&self, _start_date: &str, _end_date: &str, _api_key: Option<&str>) -> ClientResult<Value> {
            Ok(serde_json::json!([]))
        }
    }

    #[tokio::test]
    async fn test_get_osdr_items() {
        let service = OsdrServiceImpl::new(MockOsdrRepo, MockNasaClient);
        let result = service.get_osdr_items(10).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_osdr_data() {
        let service = OsdrServiceImpl::new(MockOsdrRepo, MockNasaClient);
        let result = service.sync_osdr_data("dummy_url").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}
