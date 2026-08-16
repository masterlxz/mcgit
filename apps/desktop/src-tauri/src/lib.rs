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

            app.manage(AppState { db, java_dir });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::java::scan_system_java,
            commands::java::list_java_installations,
            commands::java::list_installable_java_versions,
            commands::java::install_java,
            commands::java::add_manual_java,
            commands::java::set_default_java,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
