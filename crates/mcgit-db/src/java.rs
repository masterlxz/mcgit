use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::connection::{Db, DbError};
use crate::entities::java_installation::{ActiveModel, Column, Entity, JavaSource, Model};

/// What's needed to insert (or, if `path` already exists, update) a row —
/// `id`, `is_default`, and `created_at` are assigned by the database.
pub struct NewJavaInstallation {
    pub major_version: i32,
    pub vendor: String,
    pub path: String,
    pub source: JavaSource,
}

/// Inserts a new installation, or — if `path` already exists — updates the
/// existing row's version/vendor/source in place. `path` is the dedup key
/// shared by the detection, download, and manual-entry flows.
pub async fn upsert_by_path(db: &Db, new: NewJavaInstallation) -> Result<Model, DbError> {
    if let Some(existing) = find_by_path(db, &new.path).await? {
        let mut active: ActiveModel = existing.into();
        active.major_version = Set(new.major_version);
        active.vendor = Set(new.vendor);
        active.source = Set(new.source);
        Ok(active.update(&db.conn).await?)
    } else {
        let active = ActiveModel {
            major_version: Set(new.major_version),
            vendor: Set(new.vendor),
            path: Set(new.path),
            source: Set(new.source),
            is_default: Set(false),
            ..Default::default()
        };
        Ok(active.insert(&db.conn).await?)
    }
}

pub async fn list_all(db: &Db) -> Result<Vec<Model>, DbError> {
    Entity::find()
        .order_by_asc(Column::MajorVersion)
        .order_by_asc(Column::Vendor)
        .all(&db.conn)
        .await
        .map_err(DbError::from)
}

pub async fn find_by_path(db: &Db, path: &str) -> Result<Option<Model>, DbError> {
    Entity::find()
        .filter(Column::Path.eq(path))
        .one(&db.conn)
        .await
        .map_err(DbError::from)
}

/// Marks `id` as the default installation, clearing any previous default —
/// wrapped in a transaction so the two updates are never observed half-done.
pub async fn set_default(db: &Db, id: i64) -> Result<(), DbError> {
    db.conn
        .transaction::<_, (), DbError>(|txn| {
            Box::pin(async move {
                let target = Entity::find_by_id(id)
                    .one(txn)
                    .await?
                    .ok_or(DbError::NotFound)?;

                if let Some(current_default) = Entity::find()
                    .filter(Column::IsDefault.eq(true))
                    .one(txn)
                    .await?
                {
                    let mut active: ActiveModel = current_default.into();
                    active.is_default = Set(false);
                    active.update(txn).await?;
                }

                let mut active: ActiveModel = target.into();
                active.is_default = Set(true);
                active.update(txn).await?;

                Ok(())
            })
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => DbError::Sea(db_err),
            sea_orm::TransactionError::Transaction(db_error) => db_error,
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

    #[tokio::test]
    async fn upsert_then_list_roundtrips() {
        let db = Db::open_in_memory().await.unwrap();
        let inserted = upsert_by_path(&db, sample("/opt/java/21/bin/java"))
            .await
            .unwrap();

        assert_eq!(inserted.major_version, 21);
        assert_eq!(inserted.source, JavaSource::Managed);
        assert!(!inserted.is_default);

        let all = list_all(&db).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].path, "/opt/java/21/bin/java");
    }

    #[tokio::test]
    async fn upsert_same_path_updates_instead_of_duplicating() {
        let db = Db::open_in_memory().await.unwrap();
        upsert_by_path(&db, sample("/opt/java/21/bin/java"))
            .await
            .unwrap();

        let mut updated = sample("/opt/java/21/bin/java");
        updated.vendor = "Amazon Corretto".to_string();
        upsert_by_path(&db, updated).await.unwrap();

        let all = list_all(&db).await.unwrap();
        assert_eq!(all.len(), 1, "same path must update, not duplicate");
        assert_eq!(all[0].vendor, "Amazon Corretto");
    }

    #[tokio::test]
    async fn set_default_clears_previous_default() {
        let db = Db::open_in_memory().await.unwrap();
        let first = upsert_by_path(&db, sample("/opt/java/17/bin/java"))
            .await
            .unwrap();
        let second = upsert_by_path(&db, sample("/opt/java/21/bin/java"))
            .await
            .unwrap();

        set_default(&db, first.id).await.unwrap();
        set_default(&db, second.id).await.unwrap();

        let all = list_all(&db).await.unwrap();
        let defaults: Vec<_> = all.iter().filter(|row| row.is_default).collect();
        assert_eq!(defaults.len(), 1, "only one row may be the default");
        assert_eq!(defaults[0].id, second.id);
    }

    #[tokio::test]
    async fn set_default_on_unknown_id_errors() {
        let db = Db::open_in_memory().await.unwrap();
        let result = set_default(&db, 999).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }
}
