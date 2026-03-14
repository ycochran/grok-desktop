use tauri::{AppHandle, Manager};

use crate::{persistence::state::PersistedStateStore, window};

pub const APP_TITLE: &str = "Grok Desktop for macOS";

pub fn configure_main_window(app: &AppHandle) -> tauri::Result<()> {
    let main_window = app
        .get_webview_window("main")
        .expect("main window should be available during setup");

    let store = app.state::<PersistedStateStore>();
    window::menu::install_app_menu(app)?;
    main_window.set_title(APP_TITLE)?;
    window::restore::restore_main_window(&main_window, &store)?;
    let _ = window::zoom::apply_saved_zoom(app, &store)?;
    window::behavior::attach_main_window_listeners(app, &main_window);
    Ok(())
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    let main_window = app
        .get_webview_window("main")
        .expect("main window should be available while running");

    main_window.set_title(APP_TITLE)?;
    let _ = main_window.unminimize();
    main_window.show()?;
    main_window.set_focus()?;
    window::grok::show_embedded_surface(app);
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> tauri::Result<()> {
    let main_window = app
        .get_webview_window("main")
        .expect("main window should be available while running");

    main_window.set_title(APP_TITLE)?;
    window::grok::hide_embedded_surface(app);
    main_window.hide()?;
    Ok(())
}
