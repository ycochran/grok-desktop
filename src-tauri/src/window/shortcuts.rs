use tauri::{plugin::TauriPlugin, Wry};

use super::setup;

#[cfg(target_os = "macos")]
pub fn build_global_shortcut_plugin() -> Result<TauriPlugin<Wry>, tauri_plugin_global_shortcut::Error> {
    use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

    Ok(tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts(["cmd+shift+space"])?
        .with_handler(|app, shortcut, event| {
            if event.state == ShortcutState::Pressed
                && shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::Space)
            {
                let _ = setup::show_main_window(app);
            }
        })
        .build())
}

#[cfg(not(target_os = "macos"))]
pub fn build_global_shortcut_plugin() -> Result<TauriPlugin<Wry>, tauri_plugin_global_shortcut::Error> {
    Ok(tauri_plugin_global_shortcut::Builder::new().build())
}
