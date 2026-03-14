use tauri::{AppHandle, Emitter};

pub const COMMAND_PALETTE_EVENT: &str = "app://command-palette";

pub fn open_command_palette(app: &AppHandle) {
    open_command_palette_with_query(app, "");
}

pub fn open_command_palette_with_query(app: &AppHandle, query: &str) {
    let _ = app.emit_to("main", COMMAND_PALETTE_EVENT, query);
}
