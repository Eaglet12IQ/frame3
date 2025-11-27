use axum::{
    extract::State,
    Json,
};
use chrono::DateTime;
use serde::Serialize;
use serde_json::Value;

use crate::{AppState, domain::IssData, services::IssService};

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

pub async fn last_iss(State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    match st.iss_service.get_latest_iss_data().await {
        Ok(Some(iss_data)) => Ok(Json(serde_json::json!({
            "id": iss_data.id, "fetched_at": iss_data.fetched_at, "source_url": iss_data.source_url, "payload": iss_data.payload
        }))),
        Ok(None) => Ok(Json(serde_json::json!({"message":"no data"}))),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn trigger_iss(State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    super::fetch_and_store_iss(&st).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    last_iss(State(st)).await
}

pub async fn iss_trend(State(st): State<AppState>)
-> Result<Json<Trend>, (axum::http::StatusCode, String)> {
    let trend = st.iss_service.get_iss_trend_analysis().await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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
