use sea_orm::entity::prelude::*;

/// A generic key-value row — the first entity in this crate keyed by a
/// `String` primary key instead of an auto-increment `id` (every other
/// table here has one), since a setting's identity *is* its name.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
