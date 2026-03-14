use std::fs;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::persistence::state::{PersistedStateStore, ScratchpadState};
use crate::window::navigation;

pub const SCRATCHPAD_STATE_EVENT: &str = "app://scratchpad-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScratchpadActionResult {
    pub state: ScratchpadState,
    pub message: String,
}

#[tauri::command]
pub fn get_scratchpad_state(
    store: State<'_, PersistedStateStore>,
) -> Result<ScratchpadState, String> {
    let guard = store
        .inner
        .lock()
        .map_err(|_| String::from("settings store mutex poisoned"))?;

    Ok(guard.scratchpad.clone())
}

#[tauri::command]
pub fn update_scratchpad_state(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    scratchpad: ScratchpadState,
) -> Result<ScratchpadState, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.scratchpad = scratchpad;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_scratchpad_state(&app, &updated_state.scratchpad);
    Ok(updated_state.scratchpad)
}

#[tauri::command]
pub fn toggle_scratchpad_state(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<ScratchpadState, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.scratchpad.is_open = !guard.scratchpad.is_open;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_scratchpad_state(&app, &updated_state.scratchpad);
    Ok(updated_state.scratchpad)
}

#[tauri::command]
pub fn import_scratchpad_content(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<ScratchpadActionResult, String> {
    let Some(selected_file) = app
        .dialog()
        .file()
        .add_filter("Text", &["txt", "md"])
        .set_title("Import Scratchpad")
        .blocking_pick_file()
    else {
        return Err(String::from("Import cancelled"));
    };

    let path = selected_file
        .into_path()
        .map_err(|error| format!("Failed to resolve selected file path: {error}"))?;
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read scratchpad import file: {error}"))?;

    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.scratchpad.content = content;
        guard.scratchpad.is_open = true;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_scratchpad_state(&app, &updated_state.scratchpad);

    Ok(ScratchpadActionResult {
        state: updated_state.scratchpad,
        message: String::from("Scratchpad imported."),
    })
}

#[tauri::command]
pub fn export_scratchpad_content(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    content: Option<String>,
) -> Result<ScratchpadActionResult, String> {
    let current_scratchpad = {
        let guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.scratchpad.clone()
    };

    let Some(selected_file) = app
        .dialog()
        .file()
        .add_filter("Text", &["txt", "md"])
        .set_file_name("scratchpad.txt")
        .set_title("Export Scratchpad")
        .blocking_save_file()
    else {
        return Err(String::from("Export cancelled"));
    };

    let path = selected_file
        .into_path()
        .map_err(|error| format!("Failed to resolve export path: {error}"))?;
    let contents = content.unwrap_or(current_scratchpad.content.clone());
    fs::write(&path, contents).map_err(|error| format!("Failed to export scratchpad: {error}"))?;

    Ok(ScratchpadActionResult {
        state: current_scratchpad,
        message: String::from("Scratchpad exported."),
    })
}

#[tauri::command]
pub fn copy_scratchpad_content(_app: AppHandle, content: String) -> Result<(), String> {
    navigation::copy_to_clipboard(&content)
}

#[tauri::command]
pub fn clear_scratchpad_content(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<ScratchpadState, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.scratchpad.content.clear();
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_scratchpad_state(&app, &updated_state.scratchpad);
    Ok(updated_state.scratchpad)
}

pub(crate) fn emit_scratchpad_state(app: &AppHandle, state: &ScratchpadState) {
    let _ = app.emit_to("main", SCRATCHPAD_STATE_EVENT, state);
}
