use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use super::{menu, setup};

const TRAY_ID: &str = "main-tray";

/// Lightweight tray-icon states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayState {
    Idle,
    Attention,
}

pub fn install_system_tray(app: &AppHandle) -> tauri::Result<()> {
    let tray_menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, menu::MENU_SHOW, "Show", true, None::<&str>)?,
            &MenuItem::with_id(app, menu::MENU_HIDE, "Hide", true, None::<&str>)?,
            &MenuItem::with_id(app, menu::MENU_RELOAD_GROK, "Reload Grok", true, None::<&str>)?,
            &MenuItem::with_id(app, menu::MENU_OPEN_SETTINGS, "Open Settings", true, None::<&str>)?,
            &MenuItem::with_id(app, menu::MENU_QUIT, "Quit", true, None::<&str>)?,
        ],
    )?;

    let icon = load_icon(TrayState::Idle);

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .tooltip(setup::APP_TITLE)
        .icon_as_template(true);

    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }

    builder
        .on_tray_icon_event(|tray: &TrayIcon<_>, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = setup::show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Switch the tray icon between idle and attention states.
pub fn set_tray_state(app: &AppHandle, state: TrayState) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    if let Some(icon) = load_icon(state) {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_icon_as_template(true);
    }
}

fn load_icon(state: TrayState) -> Option<Image<'static>> {
    let bytes: &[u8] = match state {
        TrayState::Idle => include_bytes!("../../icons/tray-idle@2x.png"),
        TrayState::Attention => include_bytes!("../../icons/tray-attention@2x.png"),
    };
    Image::from_bytes(bytes).ok()
}
