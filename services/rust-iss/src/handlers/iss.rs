use axum::{
    extract::State,
    Json,
};
use chrono::DateTime;
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info, instrument};

use crate::{AppState, domain::IssData, services::IssService, handlers::ApiError};

#[derive(Serialize)]
pub struct IssResponse {
    pub id: i64,
    pub fetched_at: DateTime<chrono::Utc>,
    pub source_url: String,
    pub payload: Value,
}

#[derive(Serialize)]
pub struct Trend {
    pub movement: bool,
    pub delta_km: f64,
    pub dt_sec: f64,
    pub velocity_kmh: Option<f64>,
    pub from_time: Option<DateTime<chrono::Utc>>,
    pub to_time: Option<DateTime<chrono::Utc>>,
    pub from_lat: Option<f64>,
    pub from_lon: Option<f64>,
    pub to_lat: Option<f64>,
    pub to_lon: Option<f64>,
}

#[instrument(skip(st))]
pub async fn last_iss(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    info!("Retrieving latest ISS data");
    match st.iss_service.get_latest_iss_data().await {
        Ok(Some(iss_data)) => {
            info!("Found ISS data with id: {}", iss_data.id.unwrap_or(0));
            Ok(Json(serde_json::json!({
                "id": iss_data.id, "fetched_at": iss_data.fetched_at, "source_url": iss_data.source_url, "payload": iss_data.payload
            })))
        }
        Ok(None) => {
            info!("No ISS data found");
            Ok(Json(serde_json::json!({"message":"no data"})))
        }
        Err(e) => {
            error!("Failed to retrieve ISS data: {:?}", e);
            Err(ApiError::internal_error("Failed to retrieve ISS data"))
        }
    }
}

#[instrument(skip(st))]
pub async fn trigger_iss(State(st): State<AppState>) -> Result<Json<Value>, ApiError> {
    info!("Triggering ISS data fetch");
    super::fetch_and_store_iss(&st).await
        .map_err(|e| {
            error!("Failed to fetch and store ISS data: {:?}", e);
            ApiError::internal_error("Failed to fetch ISS data")
        })?;
    info!("ISS data fetch completed, retrieving latest data");
    last_iss(State(st)).await
}

#[instrument(skip(st))]
pub async fn iss_trend(State(st): State<AppState>) -> Result<Json<Trend>, ApiError> {
    info!("Calculating ISS trend analysis");
    let trend = st.iss_service.get_iss_trend_analysis().await
        .map_err(|e| {
            error!("Failed to get ISS trend analysis: {:?}", e);
            ApiError::internal_error("Failed to calculate ISS trend")
        })?;

    info!("ISS trend calculated: movement={}, delta_km={}", trend.movement, trend.delta_km);
    Ok(Json(Trend {
        movement: trend.movement,
        delta_km: trend.delta_km,
        dt_sec: trend.dt_sec,
        velocity_kmh: trend.velocity_kmh,
        from_time: trend.from_time,
        to_time: trend.to_time,
        from_lat: trend.from_lat,
        from_lon: trend.from_lon,
        to_lat: trend.to_lat,
        to_lon: trend.to_lon,
    }))
}
