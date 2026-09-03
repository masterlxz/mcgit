use serde::Serialize;
use tauri::State;

use mcgit_db::settings as db_settings;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct CommitIdentityDto {
    pub name: String,
    pub email: String,
}

/// The commit identity currently in effect — resolved with defaults
/// applied, so the UI always shows the real value that would be used on
/// the next commit, not an empty/unset field.
#[tauri::command]
pub async fn get_commit_identity(state: State<'_, AppState>) -> Result<CommitIdentityDto, String> {
    let (name, email) = db_settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(CommitIdentityDto { name, email })
}

/// Sets the commit identity — or, for whichever field is left blank,
/// clears it back to the built-in default (`mcgit`/`mcgit@localhost`), via
/// `set_or_clear` (never stores a blank value, which would otherwise make
/// `get_commit_identity` return it verbatim instead of falling back).
#[tauri::command]
pub async fn set_commit_identity(name: String, email: String, state: State<'_, AppState>) -> Result<(), String> {
    db_settings::set_or_clear(&state.db, db_settings::COMMIT_NAME_KEY, &name)
        .await
        .map_err(|e| e.to_string())?;
    db_settings::set_or_clear(&state.db, db_settings::COMMIT_EMAIL_KEY, &email)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
