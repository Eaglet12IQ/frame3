use std::{collections::HashMap, default::Default, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tower_http::trace::TraceLayer;

mod domain;
mod repo;
mod services;
mod clients;
mod config;
mod handlers;
mod routes;
use domain::*;
use repo::*;
use services::*;
use clients::{NasaClient, NasaClientImpl, IssClient, IssClientImpl, SpaceXClient, SpaceXClientImpl};
use config::*;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    iss_service: IssServiceImpl<PgRepos>,
    osdr_service: OsdrServiceImpl<PgRepos>,
    cache_service: CacheServiceImpl<PgRepos>,
    nasa_client: NasaClientImpl,
    iss_client: IssClientImpl,
    spacex_client: SpaceXClientImpl,
    config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;
    config.validate().map_err(|e| anyhow::anyhow!("Configuration validation error: {}", e))?;

    let pool = PgPoolOptions::new().max_connections(config.database.max_connections).connect(&config.database.url).await?;
    init_db(&pool).await?;

    let iss_repo = PgRepos::new(pool.clone());
    let osdr_repo = PgRepos::new(pool.clone());
    let cache_repo = PgRepos::new(pool.clone());

    let iss_service = IssServiceImpl::new(iss_repo);
    let osdr_service = OsdrServiceImpl::new(osdr_repo);
    let cache_service = CacheServiceImpl::new(cache_repo);

    let http_config = HttpClientConfig::default();

    let nasa_client = NasaClientImpl::new(http_config.clone());
    let iss_client = IssClientImpl::new(http_config.clone());
    let spacex_client = SpaceXClientImpl::new(http_config.clone());

    let state = AppState {
        pool: pool.clone(),
        iss_service,
        osdr_service,
        cache_service,
        nasa_client,
        iss_client,
        spacex_client,
        config: config.clone(),
    };

    // фон OSDR
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_and_store_osdr(&st).await { error!("osdr err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.osdr.fetch_interval)).await;
            }
        });
    }
    // фон ISS
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_and_store_iss(&st).await { error!("iss err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.iss.fetch_interval)).await;
            }
        });
    }
    // фон APOD
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_apod(&st).await { error!("apod err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.nasa.fetch_intervals.apod)).await;
            }
        });
    }
    // фон NeoWs
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_neo_feed(&st).await { error!("neo err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.nasa.fetch_intervals.neo)).await;
            }
        });
    }
    // фон DONKI
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_donki(&st).await { error!("donki err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.nasa.fetch_intervals.donki)).await;
            }
        });
    }
    // фон SpaceX
    {
        let st = state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = handlers::fetch_spacex_next(&st).await { error!("spacex err {e:?}") }
                tokio::time::sleep(Duration::from_secs(st.config.spacex.fetch_interval)).await;
            }
        });
    }

    let app = routes::create_routes()
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await?;
    info!("rust_iss listening on 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn env_u64(k: &str, d: u64) -> u64 {
    std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d)
}

/* ---------- DB boot ---------- */
async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    // ISS
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS iss_fetch_log(
            id BIGSERIAL PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            source_url TEXT NOT NULL,
            payload JSONB NOT NULL
        )"
    ).execute(pool).await?;

    // OSDR
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS osdr_items(
            id BIGSERIAL PRIMARY KEY,
            dataset_id TEXT,
            title TEXT,
            status TEXT,
            updated_at TIMESTAMPTZ,
            inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            raw JSONB NOT NULL
        )"
    ).execute(pool).await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
         ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL"
    ).execute(pool).await?;

    // универсальный кэш космоданных
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS space_cache(
            id BIGSERIAL PRIMARY KEY,
            source TEXT NOT NULL,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            payload JSONB NOT NULL
        )"
    ).execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS ix_space_cache_source ON space_cache(source,fetched_at DESC)").execute(pool).await?;

    Ok(())
}








