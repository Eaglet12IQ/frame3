use axum::{
    http::{HeaderMap, Request},
    routing::get,
    middleware::Next,
    response::Response,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer};
use tracing::info;
use uuid::Uuid;

use crate::handlers;
use crate::middleware::rate_limit_middleware;
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
        .route("/iss/trend/analysis", get(handlers::iss_trend_analysis))
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
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
}
