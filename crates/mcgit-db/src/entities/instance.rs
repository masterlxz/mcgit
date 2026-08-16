use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "instances")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub mc_version: String,
    pub loader: InstanceLoader,
    pub loader_version: Option<String>,
    pub java_installation_id: Option<i64>,
    pub jvm_args: Option<String>,
    pub status: InstanceStatus,
    pub created_at: String,
}

/// Only `Vanilla` exists in this slice. Kept as a typed enum (not `TEXT`)
/// from the start, matching `JavaSource`'s pattern — Phase 3's
/// Fabric/Forge/NeoForge support only needs new variants + a migration
/// widening the `CHECK` constraint, not revisiting this decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum InstanceLoader {
    #[sea_orm(string_value = "vanilla")]
    Vanilla,
}

impl InstanceLoader {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstanceLoader::Vanilla => "vanilla",
        }
    }
}

/// Vanilla install is a long, network-heavy, interruptible operation — a row
/// exists as soon as the user asks to create an instance, before any file
/// has been downloaded, so a status is needed to tell "in progress" apart
/// from "ready to use" and "failed" (never silently vanishing on error).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum InstanceStatus {
    #[sea_orm(string_value = "installing")]
    Installing,
    #[sea_orm(string_value = "ready")]
    Ready,
    #[sea_orm(string_value = "failed")]
    Failed,
}

impl InstanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstanceStatus::Installing => "installing",
            InstanceStatus::Ready => "ready",
            InstanceStatus::Failed => "failed",
        }
    }
}

/// First real relation in the project — every other entity's `Relation` has
/// been empty so far. `java_installation_id` is nullable (the row exists
/// before Java resolution completes), so `ON DELETE SET NULL`: deleting a
/// Java installation un-links instances that pointed to it instead of
/// deleting/blocking them.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::java_installation::Entity",
        from = "Column::JavaInstallationId",
        to = "super::java_installation::Column::Id",
        on_delete = "SetNull"
    )]
    JavaInstallation,
}

impl Related<super::java_installation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::JavaInstallation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
