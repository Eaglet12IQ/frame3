use async_trait::async_trait;
use serde_json::Value;

use crate::domain::*;
use crate::repo::*;
use crate::services::*;
use crate::clients::{IssClient, Result as ClientResult, ClientError};

/// Implementation of ISS Service
#[derive(Clone)]
pub struct IssServiceImpl<R: IssRepo + Clone, C: IssClient + Clone> {
    repo: R,
    client: C,
}

impl<R: IssRepo + Clone, C: IssClient + Clone> IssServiceImpl<R, C> {
    pub fn new(repo: R, client: C) -> Self {
        Self { repo, client }
    }
}

#[async_trait]
impl<R: IssRepo + Sync + Clone, C: IssClient + Clone + Sync> IssService for IssServiceImpl<R, C> {
    async fn fetch_and_store_iss_data(&self, url: &str) -> crate::services::Result<IssData> {
        // Fetch data from ISS API using the client
        let json: Value = self.client
            .fetch_iss_position_by_url(url)
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("ISS API request failed: {}", e)))?;

        // Create domain model and validate
        let iss_data = IssData::new(url.to_string(), json);
        iss_data
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Store in repository
        self.repo
            .insert_iss_data(&iss_data)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(iss_data)
    }

    async fn get_latest_iss_data(&self) -> crate::services::Result<Option<IssData>> {
        self.repo
            .get_latest_iss_data()
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))
    }

    async fn get_iss_trend_analysis(&self) -> crate::services::Result<IssTrend> {
        let iss_data_list = self.repo
            .get_iss_trend_data()
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        if iss_data_list.len() < 2 {
            return Ok(IssTrend {
                movement: false,
                delta_km: 0.0,
                dt_sec: 0.0,
                velocity_kmh: None,
                from_time: None,
                to_time: None,
                from_lat: None,
                from_lon: None,
                to_lat: None,
                to_lon: None,
            });
        }

        let t2: DateTime<Utc> = iss_data_list[0].fetched_at;
        let t1: DateTime<Utc> = iss_data_list[1].fetched_at;
        let p2: Value = iss_data_list[0].payload.clone();
        let p1: Value = iss_data_list[1].payload.clone();

        let lat1 = extract_numeric_field(&p1, "latitude");
        let lon1 = extract_numeric_field(&p1, "longitude");
        let lat2 = extract_numeric_field(&p2, "latitude");
        let lon2 = extract_numeric_field(&p2, "longitude");
        let v2 = extract_numeric_field(&p2, "velocity");

        let mut delta_km = 0.0;
        let mut movement = false;
        if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
            delta_km = haversine_distance_km(a1, o1, a2, o2);
            movement = delta_km > 0.1;
        }
        let dt_sec = (t2 - t1).num_milliseconds() as f64 / 1000.0;

        Ok(IssTrend {
            movement,
            delta_km,
            dt_sec,
            velocity_kmh: v2,
            from_time: Some(t1),
            to_time: Some(t2),
            from_lat: lat1,
            from_lon: lon1,
            to_lat: lat2,
            to_lon: lon2,
        })
    }

    async fn get_iss_trend_points(&self, limit: usize) -> crate::services::Result<Vec<crate::services::IssPoint>> {
        let iss_data_list = self.repo
            .get_iss_data_range(limit as i64)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        let mut points = Vec::new();
        for data in iss_data_list.into_iter().rev() {
            if let (Some(lat), Some(lon)) = (
                extract_numeric_field(&data.payload, "latitude"),
                extract_numeric_field(&data.payload, "longitude"),
            ) {
                points.push(crate::services::IssPoint {
                    lat,
                    lon,
                    at: data.fetched_at,
                    velocity: extract_numeric_field(&data.payload, "velocity"),
                    altitude: extract_numeric_field(&data.payload, "altitude"),
                });
            }
        }
        Ok(points)
    }

    async fn trigger_iss_fetch(&self) -> crate::services::Result<IssData> {
        // Default ISS API URL
        let default_url = "https://api.wheretheiss.at/v1/satellites/25544";

        // Fetch data from ISS API using the client
        let json: Value = self.client
            .fetch_iss_position()
            .await
            .map_err(|e| ServiceError::ExternalApiError(format!("ISS API request failed: {}", e)))?;

        // Create domain model and validate
        let iss_data = IssData::new(default_url.to_string(), json);
        iss_data
            .validate()
            .map_err(|e| ServiceError::ValidationError(e.to_string()))?;

        // Store in repository
        self.repo
            .insert_iss_data(&iss_data)
            .await
            .map_err(|e| ServiceError::RepositoryError(e.to_string()))?;

        Ok(iss_data)
    }
}

/// Extract numeric field from JSON value
fn extract_numeric_field(value: &Value, field: &str) -> Option<f64> {
    if let Some(field_value) = value.get(field) {
        if let Some(num) = field_value.as_f64() {
            return Some(num);
        }
        if let Some(str_val) = field_value.as_str() {
            return str_val.parse::<f64>().ok();
        }
    }
    None
}

/// Calculate haversine distance between two points in kilometers
fn haversine_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let rlat1 = lat1.to_radians();
    let rlat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + rlat1.cos() * rlat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::RepoError;

    // Mock repository for testing
    #[derive(Clone)]
    struct MockIssRepo {
        data: Vec<IssData>,
    }

    impl MockIssRepo {
        fn new() -> Self {
            Self { data: Vec::new() }
        }
    }

    #[async_trait]
    impl IssRepo for MockIssRepo {
        async fn insert_iss_data(&self, data: &IssData) -> crate::repo::Result<i64> {
            Ok(1)
        }

        async fn get_latest_iss_data(&self) -> crate::repo::Result<Option<IssData>> {
            Ok(self.data.last().cloned())
        }

        async fn get_iss_data_range(&self, _limit: i64) -> crate::repo::Result<Vec<IssData>> {
            Ok(self.data.clone())
        }

        async fn get_iss_trend_data(&self) -> crate::repo::Result<Vec<IssData>> {
            Ok(self.data.iter().rev().take(2).cloned().collect())
        }
    }

    #[test]
    fn test_extract_numeric_field() {
        let json = serde_json::json!({"latitude": 51.5074, "longitude": -0.1278});
        assert_eq!(extract_numeric_field(&json, "latitude"), Some(51.5074));
        assert_eq!(extract_numeric_field(&json, "longitude"), Some(-0.1278));
        assert_eq!(extract_numeric_field(&json, "missing"), None);
    }

    #[test]
    fn test_haversine_distance_km() {
        // Distance between London and Paris (approximate)
        let london_lat = 51.5074;
        let london_lon = -0.1278;
        let paris_lat = 48.8566;
        let paris_lon = 2.3522;

        let distance = haversine_distance_km(london_lat, london_lon, paris_lat, paris_lon);
        // Approximate distance should be around 344 km
        assert!((distance - 344.0).abs() < 10.0);
    }

    #[tokio::test]
    async fn test_get_iss_trend_analysis_no_data() {
        let service = IssServiceImpl::new(MockIssRepo::new());
        let result = service.get_iss_trend_analysis().await;
        assert!(result.is_ok());
        let trend = result.unwrap();
        assert!(!trend.movement);
        assert_eq!(trend.delta_km, 0.0);
    }

    #[tokio::test]
    async fn test_get_latest_iss_data() {
        let service = IssServiceImpl::new(MockIssRepo::new());
        let result = service.get_latest_iss_data().await;
        assert!(result.is_ok());
    }
}
