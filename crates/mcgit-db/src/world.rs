use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::connection::{Db, DbError};
use crate::entities::world::{ActiveModel, Column, Entity, Model};

pub async fn list_by_instance(db: &Db, instance_id: i64) -> Result<Vec<Model>, DbError> {
    Entity::find()
        .filter(Column::InstanceId.eq(instance_id))
        .all(&db.conn)
        .await
        .map_err(DbError::from)
}

async fn find_by_instance_and_folder(
    db: &Db,
    instance_id: i64,
    folder_name: &str,
) -> Result<Option<Model>, DbError> {
    Entity::find()
        .filter(Column::InstanceId.eq(instance_id))
        .filter(Column::FolderName.eq(folder_name))
        .one(&db.conn)
        .await
        .map_err(DbError::from)
}

/// Find-or-create: used by both enabling and disabling versioning for a
/// world. In practice `disable` only ever hits the update branch (the UI
/// only shows a "Disable" action once a row already exists from a prior
/// `enable`), but sharing the logic avoids duplicating the lookup.
pub async fn set_git_enabled(
    db: &Db,
    instance_id: i64,
    folder_name: &str,
    enabled: bool,
) -> Result<Model, DbError> {
    if let Some(existing) = find_by_instance_and_folder(db, instance_id, folder_name).await? {
        let mut active: ActiveModel = existing.into();
        active.git_enabled = Set(enabled);
        Ok(active.update(&db.conn).await?)
    } else {
        let active = ActiveModel {
            instance_id: Set(instance_id),
            folder_name: Set(folder_name.to_string()),
            git_enabled: Set(enabled),
            ..Default::default()
        };
        Ok(active.insert(&db.conn).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn sample_instance(db: &Db) -> crate::entities::instance::Model {
        crate::instance::create_installing(
            db,
            crate::instance::NewInstance {
                name: "Survival".to_string(),
                mc_version: "26.2".to_string(),
            },
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn set_git_enabled_creates_row_when_missing() {
        let db = Db::open_in_memory().await.unwrap();
        let instance = sample_instance(&db).await;

        let row = set_git_enabled(&db, instance.id, "World", true).await.unwrap();

        assert_eq!(row.instance_id, instance.id);
        assert_eq!(row.folder_name, "World");
        assert!(row.git_enabled);
    }

    #[tokio::test]
    async fn toggling_does_not_duplicate_row() {
        let db = Db::open_in_memory().await.unwrap();
        let instance = sample_instance(&db).await;

        set_git_enabled(&db, instance.id, "World", true).await.unwrap();
        set_git_enabled(&db, instance.id, "World", false).await.unwrap();
        set_git_enabled(&db, instance.id, "World", true).await.unwrap();

        let rows = list_by_instance(&db, instance.id).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].git_enabled);
    }

    #[tokio::test]
    async fn list_by_instance_only_returns_rows_for_that_instance() {
        let db = Db::open_in_memory().await.unwrap();
        let first = sample_instance(&db).await;
        let second = sample_instance(&db).await;

        set_git_enabled(&db, first.id, "World", true).await.unwrap();
        set_git_enabled(&db, second.id, "Other", true).await.unwrap();

        let rows = list_by_instance(&db, first.id).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].folder_name, "World");
    }

    #[tokio::test]
    async fn deleting_instance_cascades_to_worlds() {
        let db = Db::open_in_memory().await.unwrap();
        let instance = sample_instance(&db).await;
        set_git_enabled(&db, instance.id, "World", true).await.unwrap();

        crate::entities::instance::Entity::delete_by_id(instance.id)
            .exec(&db.conn)
            .await
            .unwrap();

        let rows = list_by_instance(&db, instance.id).await.unwrap();
        assert!(rows.is_empty(), "worlds row should have been cascade-deleted with its instance");
    }
}
