use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Common domain types
pub type Id = i64;
pub type Timestamp = DateTime<Utc>;

/// ISS data domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssData {
    pub id: Option<Id>,
    pub fetched_at: Timestamp,
    pub source_url: String,
    pub payload: Value,
}

impl IssData {
    /// Create a new IssData instance
    pub fn new(source_url: String, payload: Value) -> Self {
        Self {
            id: None,
            fetched_at: Utc::now(),
            source_url,
            payload,
        }
    }

    /// Validate the IssData instance
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.source_url.trim().is_empty() {
            return Err(DomainError::ValidationError("source_url cannot be empty".to_string()));
        }

        if !self.source_url.starts_with("http://") && !self.source_url.starts_with("https://") {
            return Err(DomainError::ValidationError("source_url must be a valid HTTP/HTTPS URL".to_string()));
        }

        // Validate that payload is a valid JSON object with expected ISS fields
        if let Some(obj) = self.payload.as_object() {
            let has_latitude = obj.contains_key("latitude");
            let has_longitude = obj.contains_key("longitude");

            if !has_latitude || !has_longitude {
                return Err(DomainError::ValidationError(
                    "ISS payload must contain 'latitude' and 'longitude' fields".to_string()
                ));
            }
        } else {
            return Err(DomainError::ValidationError("payload must be a JSON object".to_string()));
        }

        Ok(())
    }
}

/// OSDR item domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdrItem {
    pub id: Option<Id>,
    pub dataset_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub updated_at: Option<Timestamp>,
    pub inserted_at: Timestamp,
    pub raw: Value,
}

impl OsdrItem {
    /// Create a new OsdrItem instance
    pub fn new(raw: Value) -> Self {
        Self {
            id: None,
            dataset_id: None,
            title: None,
            status: None,
            updated_at: None,
            inserted_at: Utc::now(),
            raw,
        }
    }

    /// Create OsdrItem with extracted fields
    pub fn with_fields(
        dataset_id: Option<String>,
        title: Option<String>,
        status: Option<String>,
        updated_at: Option<Timestamp>,
        raw: Value,
    ) -> Self {
        Self {
            id: None,
            dataset_id,
            title,
            status,
            updated_at,
            inserted_at: Utc::now(),
            raw,
        }
    }

    /// Validate the OsdrItem instance
    pub fn validate(&self) -> Result<(), DomainError> {
        // Dataset ID validation if present
        if let Some(ref ds_id) = self.dataset_id {
            if ds_id.trim().is_empty() {
                return Err(DomainError::ValidationError("dataset_id cannot be empty if provided".to_string()));
            }
        }

        // Title validation if present
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err(DomainError::ValidationError("title cannot be empty if provided".to_string()));
            }
        }

        // Status validation if present - should be one of known statuses
        if let Some(ref status) = self.status {
            let valid_statuses = ["active", "inactive", "pending", "completed", "archived"];
            if !valid_statuses.contains(&status.to_lowercase().as_str()) {
                return Err(DomainError::ValidationError(format!(
                    "status '{}' is not a valid status. Valid statuses: {}",
                    status,
                    valid_statuses.join(", ")
                )));
            }
        }

        // Raw data must be a valid JSON object
        if !self.raw.is_object() {
            return Err(DomainError::ValidationError("raw must be a JSON object".to_string()));
        }

        Ok(())
    }
}

/// Space cache domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceCache {
    pub id: Option<Id>,
    pub source: String,
    pub fetched_at: Timestamp,
    pub payload: Value,
}

impl SpaceCache {
    /// Create a new SpaceCache instance
    pub fn new(source: String, payload: Value) -> Self {
        Self {
            id: None,
            source,
            fetched_at: Utc::now(),
            payload,
        }
    }

    /// Validate the SpaceCache instance
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.source.trim().is_empty() {
            return Err(DomainError::ValidationError("source cannot be empty".to_string()));
        }

        // Source should be one of the known space data sources
        let valid_sources = ["apod", "neo", "flr", "cme", "spacex"];
        if !valid_sources.contains(&self.source.to_lowercase().as_str()) {
            return Err(DomainError::ValidationError(format!(
                "source '{}' is not a valid source. Valid sources: {}",
                self.source,
                valid_sources.join(", ")
            )));
        }

        // Payload must be valid JSON
        if self.payload.is_null() {
            return Err(DomainError::ValidationError("payload cannot be null".to_string()));
        }

        Ok(())
    }
}

/// Domain validation error
#[derive(Debug, Clone)]
pub enum DomainError {
    ValidationError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Helper functions for domain validation
pub mod validators {
    use super::*;

    /// Validate that a string is not empty after trimming
    pub fn validate_non_empty_string(s: &str, field_name: &str) -> Result<(), DomainError> {
        if s.trim().is_empty() {
            return Err(DomainError::ValidationError(format!("{} cannot be empty", field_name)));
        }
        Ok(())
    }

    /// Validate that a value is a valid JSON object
    pub fn validate_json_object(value: &Value, field_name: &str) -> Result<(), DomainError> {
        if !value.is_object() {
            return Err(DomainError::ValidationError(format!("{} must be a JSON object", field_name)));
        }
        Ok(())
    }

    /// Validate URL format
    pub fn validate_url(url: &str, field_name: &str) -> Result<(), DomainError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(DomainError::ValidationError(format!("{} must be a valid HTTP/HTTPS URL", field_name)));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iss_data_validation() {
        let valid_payload = serde_json::json!({
            "latitude": 51.5074,
            "longitude": -0.1278,
            "velocity": 27500.0
        });
        let iss_data = IssData::new("https://api.example.com/iss".to_string(), valid_payload);
        assert!(iss_data.validate().is_ok());

        // Test invalid source URL
        let invalid_iss = IssData::new("".to_string(), valid_payload);
        assert!(invalid_iss.validate().is_err());

        // Test invalid payload
        let invalid_payload = serde_json::json!("not an object");
        let invalid_iss = IssData::new("https://api.example.com/iss".to_string(), invalid_payload);
        assert!(invalid_iss.validate().is_err());
    }

    #[test]
    fn test_osdr_item_validation() {
        let raw_data = serde_json::json!({"dataset_id": "123", "title": "Test Dataset"});
        let osdr_item = OsdrItem::with_fields(
            Some("123".to_string()),
            Some("Test Dataset".to_string()),
            Some("active".to_string()),
            None,
            raw_data,
        );
        assert!(osdr_item.validate().is_ok());

        // Test invalid status
        let invalid_osdr = OsdrItem::with_fields(
            Some("123".to_string()),
            Some("Test Dataset".to_string()),
            Some("invalid_status".to_string()),
            None,
            serde_json::json!({"test": "data"}),
        );
        assert!(invalid_osdr.validate().is_err());
    }

    #[test]
    fn test_space_cache_validation() {
        let payload = serde_json::json!({"data": "test"});
        let space_cache = SpaceCache::new("apod".to_string(), payload);
        assert!(space_cache.validate().is_ok());

        // Test invalid source
        let invalid_cache = SpaceCache::new("invalid_source".to_string(), serde_json::json!({"test": "data"}));
        assert!(invalid_cache.validate().is_err());
    }
}
