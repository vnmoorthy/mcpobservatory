//! Connection setup and migration runner.

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::str::FromStr;

#[derive(Clone)]
pub struct Store {
    pool: Pool<Sqlite>,
}

impl Store {
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

/// Open the database at `path`, applying migrations. Creates the file (and
/// any parent directories) if it does not exist.
pub async fn open(path: impl AsRef<Path>) -> Result<Store> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }

    let url = format!("sqlite://{}?mode=rwc", path.display());
    let opts = SqliteConnectOptions::from_str(&url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await
        .context("connect sqlite")?;

    apply_migrations(&pool).await?;
    Ok(Store { pool })
}

async fn apply_migrations(pool: &Pool<Sqlite>) -> Result<()> {
    sqlx::query(include_str!("migrations/0001_initial.sql"))
        .execute(pool)
        .await
        .context("apply migration 0001")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_and_migrates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("traces.db");
        let store = open(&path).await.unwrap();

        let row: (String,) =
            sqlx::query_as("SELECT value FROM schema_meta WHERE key = 'schema_version'")
                .fetch_one(store.pool())
                .await
                .unwrap();
        assert_eq!(row.0, "1");
    }

    #[tokio::test]
    async fn wal_mode_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.db");
        let store = open(&path).await.unwrap();
        let row: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(store.pool())
            .await
            .unwrap();
        assert_eq!(row.0.to_lowercase(), "wal");
    }
}
