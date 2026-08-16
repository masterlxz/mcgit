use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};

use crate::connection::{Db, DbError};
use crate::entities::instance::{ActiveModel, Column, Entity, InstanceLoader, InstanceStatus, Model};

/// What's needed to start creating an instance — everything else (`status`,
/// `java_installation_id`, `loader`) is decided by the create/mark-* flow,
/// not the caller.
pub struct NewInstance {
    pub name: String,
    pub mc_version: String,
}

/// Inserts a new instance row in `Installing` status — before any folder or
/// file exists on disk. Gives the scaffolding/download steps a real `id` to
/// key off of, and leaves a visible row (rather than nothing) if the
/// install fails partway through.
pub async fn create_installing(db: &Db, new: NewInstance) -> Result<Model, DbError> {
    let active = ActiveModel {
        name: Set(new.name),
        mc_version: Set(new.mc_version),
        loader: Set(InstanceLoader::Vanilla),
        loader_version: Set(None),
        java_installation_id: Set(None),
        jvm_args: Set(None),
        status: Set(InstanceStatus::Installing),
        ..Default::default()
    };
    Ok(active.insert(&db.conn).await?)
}

/// Marks an instance ready to use, recording which Java installation its
/// Vanilla install resolved to.
pub async fn mark_ready(db: &Db, id: i64, java_installation_id: i64) -> Result<Model, DbError> {
    let existing = Entity::find_by_id(id)
        .one(&db.conn)
        .await?
        .ok_or(DbError::NotFound)?;
    let mut active: ActiveModel = existing.into();
    active.status = Set(InstanceStatus::Ready);
    active.java_installation_id = Set(Some(java_installation_id));
    Ok(active.update(&db.conn).await?)
}

/// Marks an instance failed — the row and any partial folder on disk stay
/// around for diagnosis instead of disappearing.
pub async fn mark_failed(db: &Db, id: i64) -> Result<Model, DbError> {
    let existing = Entity::find_by_id(id)
        .one(&db.conn)
        .await?
        .ok_or(DbError::NotFound)?;
    let mut active: ActiveModel = existing.into();
    active.status = Set(InstanceStatus::Failed);
    Ok(active.update(&db.conn).await?)
}

pub async fn list_all(db: &Db) -> Result<Vec<Model>, DbError> {
    Entity::find()
        .order_by_desc(Column::CreatedAt)
        .order_by_desc(Column::Id)
        .all(&db.conn)
        .await
        .map_err(DbError::from)
}

pub async fn find_by_id(db: &Db, id: i64) -> Result<Option<Model>, DbError> {
    Entity::find_by_id(id).one(&db.conn).await.map_err(DbError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> NewInstance {
        NewInstance {
            name: "Survival".to_string(),
            mc_version: "26.2".to_string(),
        }
    }

    #[tokio::test]
    async fn create_then_list_roundtrips() {
        let db = Db::open_in_memory().await.unwrap();
        let created = create_installing(&db, sample()).await.unwrap();

        assert_eq!(created.status, InstanceStatus::Installing);
        assert_eq!(created.loader, InstanceLoader::Vanilla);
        assert!(created.java_installation_id.is_none());

        let all = list_all(&db).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Survival");
    }

    #[tokio::test]
    async fn mark_ready_sets_status_and_java_installation() {
        let db = Db::open_in_memory().await.unwrap();
        let created = create_installing(&db, sample()).await.unwrap();

        let java = crate::java::upsert_by_path(
            &db,
            crate::java::NewJavaInstallation {
                major_version: 25,
                vendor: "Eclipse Temurin".to_string(),
                path: "/opt/java/25/bin/java".to_string(),
                source: crate::entities::java_installation::JavaSource::Managed,
            },
        )
        .await
        .unwrap();

        let ready = mark_ready(&db, created.id, java.id).await.unwrap();
        assert_eq!(ready.status, InstanceStatus::Ready);
        assert_eq!(ready.java_installation_id, Some(java.id));
    }

    #[tokio::test]
    async fn mark_failed_sets_status() {
        let db = Db::open_in_memory().await.unwrap();
        let created = create_installing(&db, sample()).await.unwrap();

        let failed = mark_failed(&db, created.id).await.unwrap();
        assert_eq!(failed.status, InstanceStatus::Failed);
    }

    #[tokio::test]
    async fn mark_ready_on_unknown_id_errors() {
        let db = Db::open_in_memory().await.unwrap();
        let result = mark_ready(&db, 999, 1).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[tokio::test]
    async fn find_by_id_returns_none_when_missing() {
        let db = Db::open_in_memory().await.unwrap();
        let result = find_by_id(&db, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_orders_most_recently_created_first() {
        let db = Db::open_in_memory().await.unwrap();
        let first = create_installing(&db, sample()).await.unwrap();
        let mut second_new = sample();
        second_new.name = "Creative".to_string();
        let second = create_installing(&db, second_new).await.unwrap();

        let all = list_all(&db).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, second.id, "most recently created must come first");
        assert_eq!(all[1].id, first.id);
    }
}
