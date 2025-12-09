use std::{collections::HashMap, default::Default, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use tower_http::trace::TraceLayer;

mod domain;
mod repo;
mod services;
mod clients;
mod config;
mod handlers;
mod middleware;
mod routes;

use domain::*;
use repo::*;
use services::*;
use clients::{NasaClient, NasaClientImpl, IssClient, IssClientImpl, SpaceXClient, SpaceXClientImpl};
use config::*;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    redis_repo: Option<RedisRepos>,
    iss_service: IssServiceImpl<PgRepos, IssClientImpl>,
    osdr_service: OsdrServiceImpl<PgRepos, NasaClientImpl>,
    cache_service: CacheServiceImpl<PgRepos, NasaClientImpl, SpaceXClientImpl>,
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

    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    init_db(&pool).await?;

    // Initialize Redis repository (optional)
    let redis_repo = if let Some(redis_config) = &config.redis {
        match RedisRepos::new(&redis_config.url).await {
            Ok(repo) => {
                info!("Redis connected successfully");
                Some(repo)
            }
            Err(e) => {
                warn!("Failed to connect to Redis: {}. Continuing without Redis.", e);
                None
            }
        }
    } else {
        info!("Redis not configured, continuing without Redis");
        None
    };

    // Initialize repositories
    let iss_repo = PgRepos::new(pool.clone());
    let osdr_repo = PgRepos::new(pool.clone());
    let cache_repo = PgRepos::new(pool.clone());

    // Initialize HTTP clients
    let http_config = HttpClientConfig::default();
    let nasa_client = NasaClientImpl::new(http_config.clone());
    let iss_client = IssClientImpl::new(http_config.clone());
    let spacex_client = SpaceXClientImpl::new(http_config.clone());

    // Initialize services with dependency injection
    let iss_service = IssServiceImpl::new(iss_repo, iss_client.clone());
    let osdr_service = OsdrServiceImpl::new(osdr_repo, nasa_client.clone());
    let mut cache_service = CacheServiceImpl::new(cache_repo, nasa_client.clone(), spacex_client.clone());

    // Add Redis support to cache service if available
    if let Some(ref redis_repo) = redis_repo {
        cache_service = cache_service.with_redis(redis_repo.clone());
    }

    // Create application state
    let state = AppState {
        pool: pool.clone(),
        redis_repo,
        iss_service,
        osdr_service,
        cache_service,
        nasa_client: nasa_client.clone(),
        iss_client: iss_client.clone(),
        spacex_client: spacex_client.clone(),
        config: config.clone(),
    };

    // Create cancellation token for graceful shutdown
    let shutdown_token = CancellationToken::new();

    // Spawn background tasks with cancellation support
    spawn_background_tasks(state.clone(), shutdown_token.clone());

    // Create Axum app
    let app = routes::create_routes()
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("rust_iss listening on {}", addr);

    // Create server with graceful shutdown
    let server = axum::serve(listener, app.into_make_service());

    // Handle shutdown signals
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = shutdown_signal() => {
            info!("Shutdown signal received, initiating graceful shutdown...");
            shutdown_token.cancel();
        }
    }

    info!("Application shutdown complete");
    Ok(())
}

/// Spawn all background tasks with cancellation support
fn spawn_background_tasks(state: AppState, shutdown_token: CancellationToken) {
    // OSDR background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.osdr.fetch_interval));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_and_store_osdr(&st).await {
                            error!("OSDR fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("OSDR background task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // ISS background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.iss.fetch_interval));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_and_store_iss(&st).await {
                            error!("ISS fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("ISS background task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // APOD background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.nasa.fetch_intervals.apod));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_apod(&st).await {
                            error!("APOD fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("APOD background task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // NeoWs background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.nasa.fetch_intervals.neo));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_neo_feed(&st).await {
                            error!("NeoWs fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("NeoWs background task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // DONKI background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.nasa.fetch_intervals.donki));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_donki(&st).await {
                            error!("DONKI fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("DONKI background task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // SpaceX background task
    {
        let st = state.clone();
        let token = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(st.config.spacex.fetch_interval));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = handlers::fetch_spacex_next(&st).await {
                            error!("SpaceX fetch error: {:?}", e);
                        }
                    }
                    _ = token.cancelled() => {
                        info!("SpaceX background task shutting down");
                        break;
                    }
                }
            }
        });
    }
}

/// Listen for shutdown signals (SIGTERM, SIGINT)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
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
