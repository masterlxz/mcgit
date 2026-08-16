use rusqlite::{Row, params};

use crate::connection::{Db, DbError};
use crate::models::{JavaInstallation, JavaSource, NewJavaInstallation};

/// Inserts a new installation, or — if `path` already exists — updates the
/// existing row's version/vendor/source in place. `path` is the dedup key
/// shared by the detection, download, and manual-entry flows.
pub fn upsert_by_path(db: &Db, new: &NewJavaInstallation) -> Result<JavaInstallation, DbError> {
    db.conn.execute(
        "INSERT INTO java_installations (major_version, vendor, path, source)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET
            major_version = excluded.major_version,
            vendor = excluded.vendor,
            source = excluded.source",
        params![new.major_version, new.vendor, new.path, new.source.as_str()],
    )?;

    find_by_path(db, &new.path)?.ok_or(DbError::NotFound)
}

pub fn list_all(db: &Db) -> Result<Vec<JavaInstallation>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, major_version, vendor, path, source, is_default, created_at
         FROM java_installations ORDER BY major_version, vendor",
    )?;
    let rows = stmt.query_map([], row_to_installation)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(DbError::from)
}

pub fn find_by_path(db: &Db, path: &str) -> Result<Option<JavaInstallation>, DbError> {
    let mut stmt = db.conn.prepare(
        "SELECT id, major_version, vendor, path, source, is_default, created_at
         FROM java_installations WHERE path = ?1",
    )?;
    let mut rows = stmt.query_map(params![path], row_to_installation)?;
    rows.next().transpose().map_err(DbError::from)
}

/// Marks `id` as the default installation, clearing any previous default —
/// wrapped in a transaction so the two updates are never observed half-done.
pub fn set_default(db: &Db, id: i64) -> Result<(), DbError> {
    let tx = db.conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE java_installations SET is_default = 0 WHERE is_default = 1",
        [],
    )?;
    let updated = tx.execute(
        "UPDATE java_installations SET is_default = 1 WHERE id = ?1",
        params![id],
    )?;
    if updated == 0 {
        return Err(DbError::NotFound);
    }
    tx.commit()?;
    Ok(())
}

fn row_to_installation(row: &Row) -> rusqlite::Result<JavaInstallation> {
    let source_str: String = row.get(4)?;
    let source = JavaSource::parse(&source_str).unwrap_or(JavaSource::Detected);
    let is_default: i64 = row.get(5)?;

    Ok(JavaInstallation {
        id: row.get(0)?,
        major_version: row.get(1)?,
        vendor: row.get(2)?,
        path: row.get(3)?,
        source,
        is_default: is_default != 0,
        created_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(path: &str) -> NewJavaInstallation {
        NewJavaInstallation {
            major_version: 21,
            vendor: "Eclipse Temurin".to_string(),
            path: path.to_string(),
            source: JavaSource::Managed,
        }
    }

    #[test]
    fn upsert_then_list_roundtrips() {
        let db = Db::open_in_memory().unwrap();
        let inserted = upsert_by_path(&db, &sample("/opt/java/21/bin/java")).unwrap();

        assert_eq!(inserted.major_version, 21);
        assert_eq!(inserted.source, JavaSource::Managed);
        assert!(!inserted.is_default);

        let all = list_all(&db).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, "/opt/java/21/bin/java");
    }

    #[test]
    fn upsert_same_path_updates_instead_of_duplicating() {
        let db = Db::open_in_memory().unwrap();
        upsert_by_path(&db, &sample("/opt/java/21/bin/java")).unwrap();

        let mut updated = sample("/opt/java/21/bin/java");
        updated.vendor = "Amazon Corretto".to_string();
        upsert_by_path(&db, &updated).unwrap();

        let all = list_all(&db).unwrap();
        assert_eq!(all.len(), 1, "same path must update, not duplicate");
        assert_eq!(all[0].vendor, "Amazon Corretto");
    }

    #[test]
    fn set_default_clears_previous_default() {
        let db = Db::open_in_memory().unwrap();
        let first = upsert_by_path(&db, &sample("/opt/java/17/bin/java")).unwrap();
        let second = upsert_by_path(&db, &sample("/opt/java/21/bin/java")).unwrap();

        set_default(&db, first.id).unwrap();
        set_default(&db, second.id).unwrap();

        let all = list_all(&db).unwrap();
        let defaults: Vec<_> = all.iter().filter(|row| row.is_default).collect();
        assert_eq!(defaults.len(), 1, "only one row may be the default");
        assert_eq!(defaults[0].id, second.id);
    }

    #[test]
    fn set_default_on_unknown_id_errors() {
        let db = Db::open_in_memory().unwrap();
        let result = set_default(&db, 999);
        assert!(matches!(result, Err(DbError::NotFound)));
    }
}
