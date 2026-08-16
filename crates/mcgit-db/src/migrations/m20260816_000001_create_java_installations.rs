use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE java_installations (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                major_version INTEGER NOT NULL,
                vendor        TEXT NOT NULL,
                path          TEXT NOT NULL UNIQUE,
                source        TEXT NOT NULL CHECK (source IN ('managed', 'detected', 'manual')),
                is_default    INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
                created_at    TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX idx_java_installations_default
                ON java_installations(is_default)
                WHERE is_default = 1;",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE java_installations;")
            .await?;
        Ok(())
    }
}
