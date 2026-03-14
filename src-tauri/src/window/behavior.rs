use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

use crate::persistence::state::PersistedStateStore;

use super::grok;

pub fn attach_main_window_listeners(app: &AppHandle, window: &WebviewWindow) {
    let app_handle = app.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::Focused(is_focused) => {
            if *is_focused {
                grok::show_embedded_surface(&app_handle);
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            let close_behavior = current_close_behavior(&app_handle);

            match close_behavior.as_str() {
                "hide" => {
                    api.prevent_close();
                    let _ = super::setup::hide_main_window(&app_handle);
                }
                _ => {
                    api.prevent_close();
                    app_handle.exit(0);
                }
            }
        }
        _ => {}
    });
}

fn current_close_behavior(app: &AppHandle) -> String {
    let store = app.state::<PersistedStateStore>();
    let guard = store.inner.lock().expect("state store mutex poisoned");
    guard.settings.close_behavior.clone()
}
