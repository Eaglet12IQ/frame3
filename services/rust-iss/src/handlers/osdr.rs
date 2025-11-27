use axum::{
    extract::State,
    Json,
};
use chrono::DateTime;
use serde_json::Value;
use sqlx::Row;

use crate::AppState;

pub async fn osdr_sync(State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let written = super::fetch_and_store_osdr(&st).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "written": written })))
}

pub async fn osdr_list(State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let limit =  std::env::var("OSDR_LIST_LIMIT").ok()
        .and_then(|s| s.parse::<i64>().ok()).unwrap_or(20);

    let rows = sqlx::query(
        "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
         FROM osdr_items
         ORDER BY inserted_at DESC
         LIMIT $1"
    ).bind(limit).fetch_all(&st.pool).await
     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let out: Vec<Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.get::<i64,_>("id"),
            "dataset_id": r.get::<Option<String>,_>("dataset_id"),
            "title": r.get::<Option<String>,_>("title"),
            "status": r.get::<Option<String>,_>("status"),
            "updated_at": r.get::<Option<DateTime<chrono::Utc>>,_>("updated_at"),
            "inserted_at": r.get::<DateTime<chrono::Utc>, _>("inserted_at"),
            "raw": r.get::<Value,_>("raw"),
        })
    }).collect();

    Ok(Json(serde_json::json!({ "items": out })))
}
