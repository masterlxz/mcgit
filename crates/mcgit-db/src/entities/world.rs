use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "worlds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub instance_id: i64,
    pub folder_name: String,
    pub git_enabled: bool,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::instance::Entity",
        from = "Column::InstanceId",
        to = "super::instance::Column::Id",
        on_delete = "Cascade"
    )]
    Instance,
}

impl Related<super::instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Instance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
