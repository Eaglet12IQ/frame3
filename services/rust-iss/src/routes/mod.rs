use axum::{
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::handlers;
use crate::AppState;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    now: DateTime<Utc>,
}

async fn health() -> axum::Json<Health> {
    axum::Json(Health {
        status: "ok",
        now: Utc::now(),
    })
}

pub fn iss_routes() -> Router<AppState> {
    Router::new()
        .route("/last", get(handlers::last_iss))
        .route("/fetch", get(handlers::trigger_iss))
        .route("/iss/trend", get(handlers::iss_trend))
}

pub fn osdr_routes() -> Router<AppState> {
    Router::new()
        .route("/osdr/sync", get(handlers::osdr_sync))
        .route("/osdr/list", get(handlers::osdr_list))
}

pub fn cache_routes() -> Router<AppState> {
    Router::new()
        .route("/space/:src/latest", get(handlers::space_latest))
        .route("/space/refresh", get(handlers::space_refresh))
        .route("/space/summary", get(handlers::space_summary))
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(iss_routes())
        .merge(osdr_routes())
        .merge(cache_routes())
}
