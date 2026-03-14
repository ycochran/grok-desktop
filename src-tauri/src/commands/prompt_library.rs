use std::fs;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

use crate::persistence::state::{PersistedStateStore, PromptLibraryState, PromptSnippet};

pub const PROMPT_LIBRARY_STATE_EVENT: &str = "app://prompt-library-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptLibraryActionResult {
    pub state: PromptLibraryState,
    pub message: String,
}

#[tauri::command]
pub fn get_prompt_library_state(
    store: State<'_, PersistedStateStore>,
) -> Result<PromptLibraryState, String> {
    let guard = store
        .inner
        .lock()
        .map_err(|_| String::from("settings store mutex poisoned"))?;

    Ok(guard.prompt_library.clone())
}

#[tauri::command]
pub fn update_prompt_library_state(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    prompt_library: PromptLibraryState,
) -> Result<PromptLibraryState, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.prompt_library = prompt_library;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_prompt_library_state(&app, &updated_state.prompt_library);
    Ok(updated_state.prompt_library)
}

#[tauri::command]
pub fn toggle_prompt_library_state(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<PromptLibraryActionResult, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.prompt_library.is_open = !guard.prompt_library.is_open;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_prompt_library_state(&app, &updated_state.prompt_library);

    Ok(PromptLibraryActionResult {
        state: updated_state.prompt_library,
        message: String::from("Prompt library visibility updated."),
    })
}

#[tauri::command]
pub fn import_prompt_library(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<PromptLibraryActionResult, String> {
    let Some(selected_file) = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_title("Import Prompt Library")
        .blocking_pick_file()
    else {
        return Err(String::from("Import cancelled"));
    };

    let path = selected_file
        .into_path()
        .map_err(|error| format!("Failed to resolve selected file path: {error}"))?;
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read prompt library import file: {error}"))?;
    let prompts = serde_json::from_str::<Vec<PromptSnippet>>(&contents)
        .map_err(|error| format!("Failed to parse prompt library import file: {error}"))?;

    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.prompt_library.prompts = prompts;
        guard.prompt_library.is_open = true;
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_prompt_library_state(&app, &updated_state.prompt_library);

    Ok(PromptLibraryActionResult {
        state: updated_state.prompt_library,
        message: String::from("Prompt library imported."),
    })
}

#[tauri::command]
pub fn export_prompt_library(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
) -> Result<PromptLibraryActionResult, String> {
    let current_state = {
        let guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.prompt_library.clone()
    };

    let Some(selected_file) = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("prompt-library.json")
        .set_title("Export Prompt Library")
        .blocking_save_file()
    else {
        return Err(String::from("Export cancelled"));
    };

    let path = selected_file
        .into_path()
        .map_err(|error| format!("Failed to resolve export path: {error}"))?;
    let contents = serde_json::to_string_pretty(&current_state.prompts)
        .map_err(|error| format!("Failed to serialize prompt library: {error}"))?;
    fs::write(&path, contents)
        .map_err(|error| format!("Failed to export prompt library: {error}"))?;

    Ok(PromptLibraryActionResult {
        state: current_state,
        message: String::from("Prompt library exported."),
    })
}

pub(crate) fn emit_prompt_library_state(app: &AppHandle, state: &PromptLibraryState) {
    let _ = app.emit_to("main", PROMPT_LIBRARY_STATE_EVENT, state);
}
