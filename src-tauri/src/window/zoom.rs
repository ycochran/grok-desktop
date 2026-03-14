use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::persistence::state::{clamp_zoom, PersistedStateStore};

use super::grok::GROK_WEBVIEW_LABEL;

const ZOOM_STEP: f64 = 0.2;
pub const ZOOM_CHANGED_EVENT: &str = "app://zoom-changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoomChangedPayload {
    pub zoom_level: f64,
}

pub fn apply_saved_zoom(app: &AppHandle, store: &State<'_, PersistedStateStore>) -> tauri::Result<f64> {
    let zoom_level = {
        let guard = store.inner.lock().expect("state store mutex poisoned");
        if guard.settings.restore_zoom {
            guard.settings.zoom_level
        } else {
            1.0
        }
    };

    apply_zoom_level(app, store, zoom_level, false)
}

pub fn zoom_in(app: &AppHandle, store: &State<'_, PersistedStateStore>) -> tauri::Result<f64> {
    let current = current_zoom_level(store);
    apply_zoom_level(app, store, current + ZOOM_STEP, true)
}

pub fn zoom_out(app: &AppHandle, store: &State<'_, PersistedStateStore>) -> tauri::Result<f64> {
    let current = current_zoom_level(store);
    apply_zoom_level(app, store, current - ZOOM_STEP, true)
}

pub fn zoom_reset(app: &AppHandle, store: &State<'_, PersistedStateStore>) -> tauri::Result<f64> {
    apply_zoom_level(app, store, 1.0, true)
}

pub fn apply_zoom_level(
    app: &AppHandle,
    store: &State<'_, PersistedStateStore>,
    zoom_level: f64,
    persist: bool,
) -> tauri::Result<f64> {
    let zoom_level = clamp_zoom((zoom_level * 100.0).round() / 100.0);

    if let Some(main_webview) = app.get_webview("main") {
        main_webview.set_zoom(zoom_level)?;
    }

    if let Some(grok_webview) = app.get_webview(GROK_WEBVIEW_LABEL) {
        let _ = grok_webview.set_zoom(zoom_level);
    }

    if persist {
        let updated_state = {
            let mut guard = store.inner.lock().expect("state store mutex poisoned");
            guard.settings.zoom_level = zoom_level;
            guard.clone().normalized()
        };
        updated_state.save(app)?;
    }

    let _ = app.emit_to(
        "main",
        ZOOM_CHANGED_EVENT,
        ZoomChangedPayload { zoom_level },
    );

    Ok(zoom_level)
}

pub fn current_zoom_level(store: &State<'_, PersistedStateStore>) -> f64 {
    let guard = store.inner.lock().expect("state store mutex poisoned");
    guard.settings.zoom_level
}
