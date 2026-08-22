use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE worlds (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                instance_id   INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
                folder_name   TEXT NOT NULL,
                git_enabled   INTEGER NOT NULL DEFAULT 0 CHECK (git_enabled IN (0, 1)),
                created_at    TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .await?;

        conn.execute_unprepared(
            "CREATE UNIQUE INDEX idx_worlds_instance_folder ON worlds(instance_id, folder_name);",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE worlds;")
            .await?;
        Ok(())
    }
}
