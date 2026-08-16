use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

const SCHEMA: &str = include_str!("schema.sql");

/// A handle to the local mcgit SQLite database. Only stores metadata — never
/// world/mod file content, which stays on the filesystem (and, for versioned
/// worlds, inside Git).
pub struct Db {
    pub(crate) conn: Connection,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("record not found")]
    NotFound,
}

impl Db {
    /// Opens (or creates) the database file at `path` and applies the schema.
    /// Safe to call on every app startup — `CREATE TABLE IF NOT EXISTS`/
    /// `CREATE INDEX IF NOT EXISTS` make this idempotent.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// An in-memory database, used in tests so each test gets a fresh,
    /// isolated database with no real file and no cross-test pollution.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }
}
