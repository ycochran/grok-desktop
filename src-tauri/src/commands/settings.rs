use std::process::Command;

use tauri::{State, AppHandle};

use crate::persistence::state::{AppSettings, PersistedStateStore};

#[tauri::command]
pub fn get_settings(store: State<'_, PersistedStateStore>) -> Result<AppSettings, String> {
    let guard = store
        .inner
        .lock()
        .map_err(|_| String::from("settings store mutex poisoned"))?;

    Ok(guard.settings.clone().normalized())
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.settings = settings.normalized();
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    Ok(updated_state.settings)
}

#[tauri::command]
pub fn open_external_link(url: String) -> Result<(), String> {
    open_external_url(&url)
}

pub(crate) fn open_external_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(String::from("Only http and https links can be opened externally"));
    }

    Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|error| format!("Failed to open external link: {error}"))?;

    Ok(())
}
