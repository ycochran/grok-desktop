use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::window::grok::{self, GrokRuntimeState};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub fn mount_grok_webview(
    app: AppHandle,
    state: State<'_, GrokRuntimeState>,
    bounds: GrokBounds,
) -> Result<(), String> {
    grok::mount_or_update(&app, &state, bounds).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reload_grok_webview(
    app: AppHandle,
    state: State<'_, GrokRuntimeState>,
) -> Result<(), String> {
    grok::reload(&app, &state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_grok_in_browser(
    app: AppHandle,
    state: State<'_, GrokRuntimeState>,
) -> Result<(), String> {
    grok::open_current_in_browser(&app, &state).map_err(|error| error.to_string())
}
