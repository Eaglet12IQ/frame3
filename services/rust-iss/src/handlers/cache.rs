use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::DateTime;
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;

use crate::AppState;

pub async fn space_latest(Path(src): Path<String>, State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let row = sqlx::query(
        "SELECT fetched_at, payload FROM space_cache
         WHERE source = $1 ORDER BY id DESC LIMIT 1"
    ).bind(&src).fetch_optional(&st.pool).await
     .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(r) = row {
        let fetched_at: DateTime<chrono::Utc> = r.get("fetched_at");
        let payload: Value = r.get("payload");
        return Ok(Json(serde_json::json!({ "source": src, "fetched_at": fetched_at, "payload": payload })));
    }
    Ok(Json(serde_json::json!({ "source": src, "message":"no data" })))
}

pub async fn space_refresh(Query(q): Query<HashMap<String,String>>, State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let list = q.get("src").cloned().unwrap_or_else(|| "apod,neo,flr,cme,spacex".to_string());
    let mut done = Vec::new();
    for s in list.split(',').map(|x| x.trim().to_lowercase()) {
        match s.as_str() {
            "apod"   => { let _ = super::fetch_apod(&st).await;       done.push("apod"); }
            "neo"    => { let _ = super::fetch_neo_feed(&st).await;   done.push("neo"); }
            "flr"    => { let _ = super::fetch_donki_flr(&st).await;  done.push("flr"); }
            "cme"    => { let _ = super::fetch_donki_cme(&st).await;  done.push("cme"); }
            "spacex" => { let _ = super::fetch_spacex_next(&st).await; done.push("spacex"); }
            _ => {}
        }
    }
    Ok(Json(serde_json::json!({ "refreshed": done })))
}

async fn latest_from_cache(pool: &sqlx::PgPool, src: &str) -> Value {
    sqlx::query("SELECT fetched_at, payload FROM space_cache WHERE source=$1 ORDER BY id DESC LIMIT 1")
        .bind(src)
        .fetch_optional(pool).await.ok().flatten()
        .map(|r| serde_json::json!({"at": r.get::<DateTime<chrono::Utc>,_>("fetched_at"), "payload": r.get::<Value,_>("payload")}))
        .unwrap_or(serde_json::json!({}))
}

pub async fn space_summary(State(st): State<AppState>)
-> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let apod   = latest_from_cache(&st.pool, "apod").await;
    let neo    = latest_from_cache(&st.pool, "neo").await;
    let flr    = latest_from_cache(&st.pool, "flr").await;
    let cme    = latest_from_cache(&st.pool, "cme").await;
    let spacex = latest_from_cache(&st.pool, "spacex").await;

    let iss_last = sqlx::query("SELECT fetched_at,payload FROM iss_fetch_log ORDER BY id DESC LIMIT 1")
        .fetch_optional(&st.pool).await.ok().flatten()
        .map(|r| serde_json::json!({"at": r.get::<DateTime<chrono::Utc>,_>("fetched_at"), "payload": r.get::<Value,_>("payload")}))
        .unwrap_or(serde_json::json!({}));

    let osdr_count: i64 = sqlx::query("SELECT count(*) AS c FROM osdr_items")
        .fetch_one(&st.pool).await.map(|r| r.get::<i64,_>("c")).unwrap_or(0);

    Ok(Json(serde_json::json!({
        "apod": apod, "neo": neo, "flr": flr, "cme": cme, "spacex": spacex,
        "iss": iss_last, "osdr_count": osdr_count
    })))
}
