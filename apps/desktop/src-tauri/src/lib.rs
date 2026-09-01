mod commands;
mod state;

use directories::ProjectDirs;
use mcgit_db::Db;
use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let project_dirs = ProjectDirs::from("dev", "mcgit", "mcgit")
                .expect("could not resolve a data directory for this OS");
            let data_dir = project_dirs.data_dir();
            std::fs::create_dir_all(data_dir).expect("could not create mcgit data directory");

            let db = tauri::async_runtime::block_on(Db::open(&data_dir.join("mcgit.sqlite3")))
                .expect("could not open mcgit database");

            let java_dir = data_dir.join("java");
            std::fs::create_dir_all(&java_dir).expect("could not create java install directory");

            let instances_dir = data_dir.join("instances");
            std::fs::create_dir_all(&instances_dir).expect("could not create instances directory");

            let library_cache_dir = data_dir.join("cache").join("minecraft");
            std::fs::create_dir_all(&library_cache_dir)
                .expect("could not create minecraft library cache directory");

            app.manage(AppState {
                db,
                java_dir,
                instances_dir,
                library_cache_dir,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::java::scan_system_java,
            commands::java::list_java_installations,
            commands::java::list_installable_java_versions,
            commands::java::install_java,
            commands::java::add_manual_java,
            commands::java::set_default_java,
            commands::instance::list_instances,
            commands::instance::list_mc_versions,
            commands::instance::create_vanilla_instance,
            commands::world::list_worlds,
            commands::world::enable_world_versioning,
            commands::world::disable_world_versioning,
            commands::world::create_world_snapshot,
            commands::world::list_world_history,
            commands::world::restore_world_version,
            commands::world::delete_world_snapshot,
            commands::world::list_world_branches,
            commands::world::create_world_branch,
            commands::world::switch_world_branch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
