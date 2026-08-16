use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "CREATE TABLE instances (
                id                    INTEGER PRIMARY KEY AUTOINCREMENT,
                name                  TEXT NOT NULL,
                mc_version            TEXT NOT NULL,
                loader                TEXT NOT NULL DEFAULT 'vanilla' CHECK (loader IN ('vanilla')),
                loader_version        TEXT,
                java_installation_id  INTEGER REFERENCES java_installations(id) ON DELETE SET NULL,
                jvm_args              TEXT,
                status                TEXT NOT NULL DEFAULT 'installing'
                                          CHECK (status IN ('installing', 'ready', 'failed')),
                created_at            TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .await?;

        conn.execute_unprepared("CREATE INDEX idx_instances_status ON instances(status);")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE instances;")
            .await?;
        Ok(())
    }
}
