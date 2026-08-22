use serde::Serialize;
use tauri::State;

use mcgit_instance::{scaffold, worlds};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct WorldDto {
    pub folder_name: String,
}

impl From<worlds::World> for WorldDto {
    fn from(world: worlds::World) -> Self {
        Self {
            folder_name: world.folder_name,
        }
    }
}

/// Lists the worlds (`saves/*` subdirectories containing a `level.dat`)
/// inside an instance. Pure filesystem read, keyed by the instance's own
/// id (which is also its folder name) — no DB lookup needed, no `worlds`
/// table exists yet.
#[tauri::command]
pub async fn list_worlds(
    instance_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<WorldDto>, String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let root = scaffold::instance_root(&instances_dir, instance_id);
        worlds::list_worlds(&root)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
    .map(|worlds| worlds.into_iter().map(WorldDto::from).collect())
}
