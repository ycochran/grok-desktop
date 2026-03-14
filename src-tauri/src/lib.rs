mod commands;
mod persistence;
mod window;

use persistence::state::{PersistedState, PersistedStateStore};
use tauri::Manager;
use window::grok::GrokRuntimeState;
use window::navigation::NavigationDiagnosticsStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(PersistedStateStore::new())
        .manage(GrokRuntimeState::default())
        .manage(NavigationDiagnosticsStore::default())
        .plugin(
            window::shortcuts::build_global_shortcut_plugin()
                .expect("global shortcut plugin should be configured"),
        )
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = window::setup::show_main_window(app);
        }))
        .invoke_handler(tauri::generate_handler![
            commands::diagnostics::get_recent_navigation_hints,
            commands::diagnostics::clear_recent_navigation_hints,
            commands::diagnostics::copy_recent_navigation_hints,
            commands::prompt_library::get_prompt_library_state,
            commands::prompt_library::update_prompt_library_state,
            commands::prompt_library::toggle_prompt_library_state,
            commands::prompt_library::import_prompt_library,
            commands::prompt_library::export_prompt_library,
            commands::workspaces::get_workspace_state,
            commands::workspaces::create_workspace,
            commands::workspaces::rename_workspace,
            commands::workspaces::switch_workspace,
            commands::workspaces::delete_workspace,
            commands::scratchpad::get_scratchpad_state,
            commands::scratchpad::update_scratchpad_state,
            commands::scratchpad::toggle_scratchpad_state,
            commands::scratchpad::import_scratchpad_content,
            commands::scratchpad::export_scratchpad_content,
            commands::scratchpad::copy_scratchpad_content,
            commands::scratchpad::clear_scratchpad_content,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::open_external_link,
            commands::zoom::zoom_in,
            commands::zoom::zoom_out,
            commands::zoom::zoom_reset,
            commands::zoom::apply_saved_zoom,
            commands::grok::mount_grok_webview,
            commands::grok::reload_grok_webview,
            commands::grok::open_grok_in_browser,
        ])
        .setup(|app| {
            {
                let store = app.state::<PersistedStateStore>();
                let mut guard = store.inner.lock().expect("state store mutex poisoned");
                *guard = PersistedState::load(app.handle())?;
            }

            window::setup::configure_main_window(app.handle())?;
            window::tray::install_system_tray(app.handle())?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            window::menu::handle_menu_event(app, event.id().0.as_ref());
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = event {
            let _ = window::setup::show_main_window(app);
        }
    });
}
