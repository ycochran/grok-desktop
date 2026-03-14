use std::sync::Mutex;

use serde::Serialize;
use tauri::{
    webview::{NewWindowResponse, WebviewBuilder},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl,
};
use url::Url;

use crate::{
    commands::{grok::GrokBounds, settings},
    persistence::state::PersistedStateStore,
};

use super::{navigation, zoom};

pub const GROK_WEBVIEW_LABEL: &str = "grok-surface";
pub const GROK_STATE_EVENT: &str = "app://grok-surface-state";
const GROK_HOME_URL: &str = "https://grok.com/";

#[derive(Debug, Clone)]
struct GrokSessionState {
    current_url: String,
    intended_url: String,
    last_bounds: Option<GrokBounds>,
    auth_in_progress: bool,
    auth_refresh_requested: bool,
}

impl Default for GrokSessionState {
    fn default() -> Self {
        Self {
            current_url: String::from(GROK_HOME_URL),
            intended_url: String::from(GROK_HOME_URL),
            last_bounds: None,
            auth_in_progress: false,
            auth_refresh_requested: false,
        }
    }
}

#[derive(Default)]
pub struct GrokRuntimeState {
    inner: Mutex<GrokSessionState>,
}

impl GrokRuntimeState {
    pub fn current_url(&self) -> String {
        self.inner
            .lock()
            .expect("grok state mutex poisoned")
            .current_url
            .clone()
    }

    pub fn remember_bounds(&self, bounds: GrokBounds) {
        let mut guard = self.inner.lock().expect("grok state mutex poisoned");
        guard.last_bounds = Some(bounds);
    }

    pub fn last_bounds(&self) -> Option<GrokBounds> {
        self.inner
            .lock()
            .expect("grok state mutex poisoned")
            .last_bounds
            .clone()
    }

    pub fn auth_flow_active(&self) -> bool {
        self.inner
            .lock()
            .expect("grok state mutex poisoned")
            .auth_in_progress
    }

    pub fn cancel_auth_flow(&self) {
        let mut guard = self.inner.lock().expect("grok state mutex poisoned");
        guard.auth_in_progress = false;
        guard.auth_refresh_requested = false;
    }

    fn apply_navigation(&self, url: &Url, decision: navigation::NavigationDecision) -> bool {
        let mut guard = self.inner.lock().expect("grok state mutex poisoned");
        let was_auth_in_progress = guard.auth_in_progress;
        guard.current_url = url.to_string();

        if navigation::is_embeddable_content(url) {
            guard.intended_url = url.to_string();
        }

        if decision.is_auth_flow {
            if !was_auth_in_progress {
                guard.auth_refresh_requested = false;
            }
            guard.auth_in_progress = true;
            return false;
        }

        if was_auth_in_progress && navigation::is_auth_completion_landing(url, decision) {
            guard.auth_in_progress = false;
            guard.auth_refresh_requested = false;
            return true;
        }

        false
    }

    fn consume_post_auth_reload_target(
        &self,
        url: &Url,
        decision: navigation::NavigationDecision,
    ) -> Option<String> {
        let mut guard = self.inner.lock().expect("grok state mutex poisoned");
        if !guard.auth_in_progress || guard.auth_refresh_requested {
            return None;
        }

        if navigation::should_trigger_post_auth_reload(url, decision) {
            guard.auth_refresh_requested = true;
            return Some(guard.intended_url.clone());
        }

        None
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrokSurfacePayload {
    phase: &'static str,
    url: String,
    message: String,
    is_auth_flow: bool,
}

pub fn mount_or_update(
    app: &AppHandle,
    state: &State<'_, GrokRuntimeState>,
    bounds: GrokBounds,
) -> tauri::Result<()> {
    state.remember_bounds(bounds.clone());

    if let Some(existing) = app.get_webview(GROK_WEBVIEW_LABEL) {
        existing.set_position(LogicalPosition::new(bounds.x, bounds.y))?;
        existing.set_size(LogicalSize::new(bounds.width, bounds.height))?;
        existing.show()?;
        emit_surface_state(
            app,
            state,
            "finished",
            &state.current_url(),
            "Embedded Grok is ready.",
            false,
        );
        return Ok(());
    }

    create_child_webview(app, state, &bounds)
}

pub fn reload(app: &AppHandle, state: &State<'_, GrokRuntimeState>) -> tauri::Result<()> {
    if let Some(webview) = app.get_webview(GROK_WEBVIEW_LABEL) {
        webview.reload()?;
        return Ok(());
    }

    let bounds = state.last_bounds().unwrap_or(GrokBounds {
        x: 0.0,
        y: 0.0,
        width: 1100.0,
        height: 760.0,
    });

    emit_surface_state(
        app,
        state,
        "recovering",
        GROK_HOME_URL,
        "Recovering the embedded Grok surface after it was unloaded.",
        false,
    );

    create_child_webview(app, state, &bounds)
}

pub fn open_current_in_browser(
    _app: &AppHandle,
    state: &State<'_, GrokRuntimeState>,
) -> Result<(), String> {
    state.cancel_auth_flow();
    settings::open_external_url(&state.current_url())
}

pub fn show_embedded_surface(app: &AppHandle) {
    if let Some(webview) = app.get_webview(GROK_WEBVIEW_LABEL) {
        let _ = webview.show();
        let store = app.state::<PersistedStateStore>();
        let _ = zoom::apply_saved_zoom(app, &store);
    }
}

pub fn hide_embedded_surface(app: &AppHandle) {
    if let Some(webview) = app.get_webview(GROK_WEBVIEW_LABEL) {
        let _ = webview.hide();
    }
}

fn create_child_webview(
    app: &AppHandle,
    state: &State<'_, GrokRuntimeState>,
    bounds: &GrokBounds,
) -> tauri::Result<()> {
    let main_window = app
        .get_window("main")
        .expect("main window should be available before creating the Grok webview");

    let app_handle = app.clone();
    let nav_handle = app.clone();
    let popup_handle = app.clone();
    let current_zoom = {
        let store = app.state::<PersistedStateStore>();
        zoom::current_zoom_level(&store)
    };

    emit_surface_state(
        app,
        state,
        "started",
        GROK_HOME_URL,
        "Loading Grok inside the embedded surface.",
        false,
    );

    let webview = main_window.add_child(
        WebviewBuilder::new(
            GROK_WEBVIEW_LABEL,
            WebviewUrl::External(Url::parse(GROK_HOME_URL).expect("grok url should parse")),
        )
        .on_navigation(move |url| handle_navigation_attempt(&nav_handle, url, false))
        .on_new_window(move |url, _features| {
            handle_navigation_attempt(&popup_handle, &url, true);
            NewWindowResponse::Deny
        })
        .on_page_load(move |webview, payload| {
            if let Some(runtime_state) = app_handle.try_state::<GrokRuntimeState>() {
                let auth_flow_active = runtime_state.auth_flow_active();
                let decision = navigation::classify_with_auth_context(payload.url(), auth_flow_active);
                navigation::log_decision(
                    &app_handle,
                    "page_load",
                    payload.url(),
                    decision,
                    auth_flow_active,
                );
                let auth_completed = runtime_state.apply_navigation(payload.url(), decision);
                if auth_completed {
                    navigation::record_auth_landing(
                        &app_handle,
                        payload.url(),
                        "Embedded sign-in completed and landed in Grok.",
                    );
                }

                if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) {
                    if let Some(target_url) =
                        runtime_state.consume_post_auth_reload_target(payload.url(), decision)
                    {
                        if let Ok(parsed_target_url) = Url::parse(&target_url) {
                            navigation::record_auth_landing(
                                &app_handle,
                                &parsed_target_url,
                                "Embedded auth bridge completed; reloading Grok with the new session.",
                            );
                            let _ = webview.navigate(parsed_target_url);
                        }
                    }
                }

                let phase = match payload.event() {
                    tauri::webview::PageLoadEvent::Started => {
                        if decision.is_auth_flow {
                            "auth"
                        } else {
                            "started"
                        }
                    }
                    tauri::webview::PageLoadEvent::Finished => {
                        if decision.is_auth_flow {
                            "auth"
                        } else {
                            "finished"
                        }
                    }
                };

                emit_surface_state(
                    &app_handle,
                    &runtime_state,
                    phase,
                    payload.url().as_str(),
                    decision.message,
                    decision.is_auth_flow,
                );
            }
        }),
        LogicalPosition::new(bounds.x, bounds.y),
        LogicalSize::new(bounds.width, bounds.height),
    )?;

    webview.set_zoom(current_zoom)?;
    Ok(())
}

fn handle_navigation_attempt(app: &AppHandle, url: &Url, popup_request: bool) -> bool {
    if let Some(runtime_state) = app.try_state::<GrokRuntimeState>() {
        let auth_flow_active = runtime_state.auth_flow_active();
        let decision = navigation::classify_with_auth_context(url, auth_flow_active);
        navigation::log_decision(
            app,
            if popup_request { "popup_request" } else { "navigation" },
            url,
            decision,
            auth_flow_active,
        );
        let auth_completed = runtime_state.apply_navigation(url, decision);
        if auth_completed {
            navigation::record_auth_landing(
                app,
                url,
                "Embedded sign-in completed after consent/account redirect.",
            );
        }

        match decision.disposition {
            navigation::NavigationDisposition::Embed => {
                if popup_request {
                    if let Some(webview) = app.get_webview(GROK_WEBVIEW_LABEL) {
                        let _ = webview.navigate(url.clone());
                    }
                }

                emit_surface_state(
                    app,
                    &runtime_state,
                    if decision.is_auth_flow { "auth" } else { "started" },
                    url.as_str(),
                    decision.message,
                    decision.is_auth_flow,
                );
                true
            }
            navigation::NavigationDisposition::ExternalBrowser => {
                let _ = settings::open_external_url(url.as_str());
                emit_surface_state(
                    app,
                    &runtime_state,
                    "blocked",
                    url.as_str(),
                    decision.message,
                    false,
                );
                false
            }
            navigation::NavigationDisposition::Deny => {
                emit_surface_state(
                    app,
                    &runtime_state,
                    "blocked",
                    url.as_str(),
                    decision.message,
                    false,
                );
                false
            }
        }
    } else {
        false
    }
}

fn emit_surface_state(
    app: &AppHandle,
    state: &State<'_, GrokRuntimeState>,
    phase: &'static str,
    url: &str,
    message: &str,
    is_auth_flow: bool,
) {
    let _ = app.emit_to(
        "main",
        GROK_STATE_EVENT,
        GrokSurfacePayload {
            phase,
            url: url.to_string(),
            message: message.to_string(),
            is_auth_flow,
        },
    );
    let _ = state;

    // Update tray icon to reflect surface health.
    let tray_state = match phase {
        "blocked" | "auth" | "recovering" => super::tray::TrayState::Attention,
        _ => super::tray::TrayState::Idle,
    };
    super::tray::set_tray_state(app, tray_state);
}
