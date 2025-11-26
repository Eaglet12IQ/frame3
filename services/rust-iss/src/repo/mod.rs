use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::result;

use crate::domain::*;

/// Common repository error type
#[derive(Debug, Clone)]
pub enum RepoError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    Conflict(String),
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            RepoError::NotFound(msg) => write!(f, "Not found: {}", msg),
            RepoError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RepoError::Conflict(msg) => write!(f, "Conflict: {}", msg),
        }
    }
}

impl std::error::Error for RepoError {}

pub type Result<T> = result::Result<T, RepoError>;

/// ISS Repository trait
#[async_trait]
pub trait IssRepo {
    async fn insert_iss_data(&self, data: &IssData) -> Result<i64>;
    async fn get_latest_iss_data(&self) -> Result<Option<IssData>>;
    async fn get_iss_data_range(&self, limit: i64) -> Result<Vec<IssData>>;
    async fn get_iss_trend_data(&self) -> Result<Vec<IssData>>;
}

/// OSDR Repository trait
#[async_trait]
pub trait OsdrRepo {
    async fn insert_or_update_osdr_item(&self, item: &OsdrItem) -> Result<i64>;
    async fn get_osdr_items(&self, limit: i64) -> Result<Vec<OsdrItem>>;
    async fn get_osdr_item_by_id(&self, dataset_id: &str) -> Result<Option<OsdrItem>>;
    async fn count_osdr_items(&self) -> Result<i64>;
}

/// Cache Repository trait
#[async_trait]
pub trait CacheRepo {
    async fn insert_cache_entry(&self, entry: &SpaceCache) -> Result<i64>;
    async fn get_latest_cache_entry(&self, source: &str) -> Result<Option<SpaceCache>>;
    async fn get_cache_entries(&self, source: &str, limit: i64) -> Result<Vec<SpaceCache>>;
}

/// PostgreSQL implementation of repositories
#[derive(Clone)]
pub struct PgRepos {
    pool: PgPool,
}

impl PgRepos {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IssRepo for PgRepos {
    async fn insert_iss_data(&self, data: &IssData) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2) RETURNING id"
        )
        .bind(&data.source_url)
        .bind(&data.payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_latest_iss_data(&self) -> Result<Option<IssData>> {
        let row_opt = sqlx::query(
            "SELECT id, fetched_at, source_url, payload FROM iss_fetch_log ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        if let Some(row) = row_opt {
            let data = IssData {
                id: Some(row.get("id")),
                fetched_at: row.get("fetched_at"),
                source_url: row.get("source_url"),
                payload: row.get("payload"),
            };
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    async fn get_iss_data_range(&self, limit: i64) -> Result<Vec<IssData>> {
        let rows = sqlx::query(
            "SELECT id, fetched_at, source_url, payload FROM iss_fetch_log ORDER BY id DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let data = IssData {
                id: Some(row.get("id")),
                fetched_at: row.get("fetched_at"),
                source_url: row.get("source_url"),
                payload: row.get("payload"),
            };
            results.push(data);
        }
        Ok(results)
    }

    async fn get_iss_trend_data(&self) -> Result<Vec<IssData>> {
        self.get_iss_data_range(2).await
    }
}

#[async_trait]
impl OsdrRepo for PgRepos {
    async fn insert_or_update_osdr_item(&self, item: &OsdrItem) -> Result<i64> {
        let row = if let Some(dataset_id) = &item.dataset_id {
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)
                 ON CONFLICT (dataset_id) DO UPDATE
                 SET title=EXCLUDED.title, status=EXCLUDED.status,
                     updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw
                 RETURNING id"
            )
            .bind(dataset_id)
            .bind(&item.title)
            .bind(&item.status)
            .bind(&item.updated_at)
            .bind(&item.raw)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)
                 RETURNING id"
            )
            .bind::<Option<String>>(None)
            .bind(&item.title)
            .bind(&item.status)
            .bind(&item.updated_at)
            .bind(&item.raw)
            .fetch_one(&self.pool)
            .await
        }
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_osdr_items(&self, limit: i64) -> Result<Vec<OsdrItem>> {
        let rows = sqlx::query(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             ORDER BY inserted_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let item = OsdrItem {
                id: Some(row.get("id")),
                dataset_id: row.get("dataset_id"),
                title: row.get("title"),
                status: row.get("status"),
                updated_at: row.get("updated_at"),
                inserted_at: row.get("inserted_at"),
                raw: row.get("raw"),
            };
            results.push(item);
        }
        Ok(results)
    }

    async fn get_osdr_item_by_id(&self, dataset_id: &str) -> Result<Option<OsdrItem>> {
        let row_opt = sqlx::query(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             WHERE dataset_id = $1"
        )
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        if let Some(row) = row_opt {
            let item = OsdrItem {
                id: Some(row.get("id")),
                dataset_id: row.get("dataset_id"),
                title: row.get("title"),
                status: row.get("status"),
                updated_at: row.get("updated_at"),
                inserted_at: row.get("inserted_at"),
                raw: row.get("raw"),
            };
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    async fn count_osdr_items(&self) -> Result<i64> {
        let row = sqlx::query("SELECT count(*) AS c FROM osdr_items")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        Ok(row.get::<i64, _>("c"))
    }
}

#[async_trait]
impl CacheRepo for PgRepos {
    async fn insert_cache_entry(&self, entry: &SpaceCache) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO space_cache(source, payload) VALUES ($1,$2) RETURNING id"
        )
        .bind(&entry.source)
        .bind(&entry.payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_latest_cache_entry(&self, source: &str) -> Result<Option<SpaceCache>> {
        let row_opt = sqlx::query(
            "SELECT id, source, fetched_at, payload FROM space_cache
             WHERE source = $1 ORDER BY id DESC LIMIT 1"
        )
        .bind(source)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        if let Some(row) = row_opt {
            let entry = SpaceCache {
                id: Some(row.get("id")),
                source: row.get("source"),
                fetched_at: row.get("fetched_at"),
                payload: row.get("payload"),
            };
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn get_cache_entries(&self, source: &str, limit: i64) -> Result<Vec<SpaceCache>> {
        let rows = sqlx::query(
            "SELECT id, source, fetched_at, payload FROM space_cache
             WHERE source = $1 ORDER BY id DESC LIMIT $2"
        )
        .bind(source)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let entry = SpaceCache {
                id: Some(row.get("id")),
                source: row.get("source"),
                fetched_at: row.get("fetched_at"),
                payload: row.get("payload"),
            };
            results.push(entry);
        }
        Ok(results)
    }
}
