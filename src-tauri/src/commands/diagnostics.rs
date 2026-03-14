use tauri::{AppHandle, State};

use crate::window::navigation::{self, NavigationDiagnosticsStore, NavigationHintRecord};

#[tauri::command]
pub fn get_recent_navigation_hints(
    store: State<'_, NavigationDiagnosticsStore>,
) -> Result<Vec<NavigationHintRecord>, String> {
    Ok(store.recent())
}

#[tauri::command]
pub fn clear_recent_navigation_hints(
    store: State<'_, NavigationDiagnosticsStore>,
) -> Result<(), String> {
    store.clear();
    Ok(())
}

#[tauri::command]
pub fn copy_recent_navigation_hints(
    app: AppHandle,
    store: State<'_, NavigationDiagnosticsStore>,
) -> Result<String, String> {
    let contents = navigation::format_hint_records(&store.recent());
    let _ = app;
    navigation::copy_to_clipboard(&contents)?;
    Ok(contents)
}
