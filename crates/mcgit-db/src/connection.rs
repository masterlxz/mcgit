use std::path::Path;

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use thiserror::Error;

use crate::migrations::Migrator;

/// A handle to the local mcgit SQLite database. Only stores metadata — never
/// world/mod file content, which stays on the filesystem (and, for versioned
/// worlds, inside Git). `DatabaseConnection` is internally a connection pool
/// (cheap to `.clone()`, safe to share across concurrent async tasks) — no
/// `Mutex` needed around it, unlike the raw `rusqlite::Connection` this
/// replaced.
#[derive(Clone)]
pub struct Db {
    pub(crate) conn: DatabaseConnection,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sea(#[from] DbErr),
    #[error("record not found")]
    NotFound,
}

impl Db {
    /// Opens (or creates) the database file at `path` and applies any
    /// pending migrations. Safe to call on every app startup — SeaORM tracks
    /// which migrations already ran, so this is idempotent.
    pub async fn open(path: &Path) -> Result<Self, DbError> {
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let conn = Database::connect(url).await?;
        Migrator::up(&conn, None).await?;
        Ok(Self { conn })
    }

    /// An in-memory database, used in tests so each test gets a fresh,
    /// isolated database with no real file and no cross-test pollution.
    pub async fn open_in_memory() -> Result<Self, DbError> {
        let conn = Database::connect("sqlite::memory:").await?;
        Migrator::up(&conn, None).await?;
        Ok(Self { conn })
    }
}
