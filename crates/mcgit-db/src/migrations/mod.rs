mod m20260816_000001_create_java_installations;
mod m20260816_000002_create_instances;
mod m20260822_000003_create_worlds;

pub struct Migrator;

#[async_trait::async_trait]
impl sea_orm_migration::MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(m20260816_000001_create_java_installations::Migration),
            Box::new(m20260816_000002_create_instances::Migration),
            Box::new(m20260822_000003_create_worlds::Migration),
        ]
    }
}
