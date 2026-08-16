use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "java_installations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub major_version: i32,
    pub vendor: String,
    #[sea_orm(unique)]
    pub path: String,
    pub source: JavaSource,
    pub is_default: bool,
    pub created_at: String,
}

/// Stored as TEXT in SQLite. Unlike the hand-written `JavaSource::parse` this
/// replaced, an unrecognized string in the database is now a real error
/// instead of silently falling back to `Detected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum JavaSource {
    #[sea_orm(string_value = "managed")]
    Managed,
    #[sea_orm(string_value = "detected")]
    Detected,
    #[sea_orm(string_value = "manual")]
    Manual,
}

impl JavaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            JavaSource::Managed => "managed",
            JavaSource::Detected => "detected",
            JavaSource::Manual => "manual",
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
