use tauri::{AppHandle, State};

use crate::{persistence::state::PersistedStateStore, window::zoom};

#[tauri::command]
pub fn zoom_in(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<f64, String> {
    zoom::zoom_in(&app, &store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn zoom_out(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<f64, String> {
    zoom::zoom_out(&app, &store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn zoom_reset(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<f64, String> {
    zoom::zoom_reset(&app, &store).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn apply_saved_zoom(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<f64, String> {
    zoom::apply_saved_zoom(&app, &store).map_err(|error| error.to_string())
}
