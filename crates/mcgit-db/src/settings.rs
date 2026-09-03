use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::connection::{Db, DbError};
use crate::entities::setting::{ActiveModel, Entity};

pub const COMMIT_NAME_KEY: &str = "commit_name";
pub const COMMIT_EMAIL_KEY: &str = "commit_email";

/// Used whenever a key has never been set — the same identity mcgit always
/// used before this setting existed, so an unconfigured app behaves exactly
/// as it did before.
const DEFAULT_COMMIT_NAME: &str = "mcgit";
const DEFAULT_COMMIT_EMAIL: &str = "mcgit@localhost";

pub async fn get(db: &Db, key: &str) -> Result<Option<String>, DbError> {
    Ok(Entity::find_by_id(key).one(&db.conn).await?.map(|m| m.value))
}

/// Inserts `key`, or updates its value if it already exists.
pub async fn set(db: &Db, key: &str, value: &str) -> Result<(), DbError> {
    if let Some(existing) = Entity::find_by_id(key).one(&db.conn).await? {
        let mut active: ActiveModel = existing.into();
        active.value = Set(value.to_string());
        active.update(&db.conn).await?;
    } else {
        ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
        }
        .insert(&db.conn)
        .await?;
    }
    Ok(())
}

pub async fn delete(db: &Db, key: &str) -> Result<(), DbError> {
    Entity::delete_by_id(key).exec(&db.conn).await?;
    Ok(())
}

/// Sets `key` to `value` — unless `value` is blank (after trimming), in
/// which case `key` is cleared instead. Storing an empty string rather
/// than deleting the key would make `get` return `Some("")` instead of
/// `None`, so a resolver like `get_commit_identity` would never fall back
/// to its default — this is the one function that should be used from a
/// "leave blank to reset to default" form field.
pub async fn set_or_clear(db: &Db, key: &str, value: &str) -> Result<(), DbError> {
    let value = value.trim();
    if value.is_empty() {
        delete(db, key).await
    } else {
        set(db, key, value).await
    }
}

/// The commit identity to use right now, with defaults applied for
/// whichever half (or both) was never configured. Shared by the Tauri
/// command that loads current settings for the UI and every call site that
/// needs an identity before committing, so the two can never disagree.
pub async fn get_commit_identity(db: &Db) -> Result<(String, String), DbError> {
    let name = get(db, COMMIT_NAME_KEY)
        .await?
        .unwrap_or_else(|| DEFAULT_COMMIT_NAME.to_string());
    let email = get(db, COMMIT_EMAIL_KEY)
        .await?
        .unwrap_or_else(|| DEFAULT_COMMIT_EMAIL.to_string());
    Ok((name, email))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_on_unset_key_returns_none() {
        let db = Db::open_in_memory().await.unwrap();
        assert_eq!(get(&db, "nope").await.unwrap(), None);
    }

    #[tokio::test]
    async fn set_then_get_roundtrips() {
        let db = Db::open_in_memory().await.unwrap();
        set(&db, "greeting", "hello").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), Some("hello".to_string()));
    }

    #[tokio::test]
    async fn set_same_key_updates_instead_of_erroring() {
        let db = Db::open_in_memory().await.unwrap();
        set(&db, "greeting", "hello").await.unwrap();
        set(&db, "greeting", "goodbye").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), Some("goodbye".to_string()));
    }

    #[tokio::test]
    async fn delete_removes_the_key() {
        let db = Db::open_in_memory().await.unwrap();
        set(&db, "greeting", "hello").await.unwrap();
        delete(&db, "greeting").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), None);
    }

    #[tokio::test]
    async fn delete_on_unset_key_does_not_error() {
        let db = Db::open_in_memory().await.unwrap();
        delete(&db, "nope").await.unwrap();
    }

    #[tokio::test]
    async fn get_commit_identity_falls_back_to_mcgit_defaults_when_unset() {
        let db = Db::open_in_memory().await.unwrap();
        let (name, email) = get_commit_identity(&db).await.unwrap();
        assert_eq!(name, "mcgit");
        assert_eq!(email, "mcgit@localhost");
    }

    #[tokio::test]
    async fn get_commit_identity_returns_configured_values() {
        let db = Db::open_in_memory().await.unwrap();
        set(&db, COMMIT_NAME_KEY, "Alex").await.unwrap();
        set(&db, COMMIT_EMAIL_KEY, "alex@example.com").await.unwrap();

        let (name, email) = get_commit_identity(&db).await.unwrap();
        assert_eq!(name, "Alex");
        assert_eq!(email, "alex@example.com");
    }

    #[tokio::test]
    async fn set_or_clear_with_a_value_sets_it() {
        let db = Db::open_in_memory().await.unwrap();
        set_or_clear(&db, "greeting", "hello").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), Some("hello".to_string()));
    }

    #[tokio::test]
    async fn set_or_clear_with_blank_deletes_the_key_instead_of_storing_empty() {
        let db = Db::open_in_memory().await.unwrap();
        set(&db, "greeting", "hello").await.unwrap();
        set_or_clear(&db, "greeting", "   ").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), None);
    }

    #[tokio::test]
    async fn set_or_clear_with_blank_on_unset_key_does_not_error() {
        let db = Db::open_in_memory().await.unwrap();
        set_or_clear(&db, "greeting", "").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), None);
    }

    #[tokio::test]
    async fn set_or_clear_trims_before_storing() {
        let db = Db::open_in_memory().await.unwrap();
        set_or_clear(&db, "greeting", "  hello  ").await.unwrap();
        assert_eq!(get(&db, "greeting").await.unwrap(), Some("hello".to_string()));
    }
}
