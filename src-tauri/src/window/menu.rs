use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem, Submenu}, AppHandle, Manager};

use crate::{commands::{prompt_library, scratchpad}, window::{grok, setup, zoom}};

const MENU_ZOOM_IN: &str = "view.zoom_in";
const MENU_ZOOM_OUT: &str = "view.zoom_out";
const MENU_ZOOM_RESET: &str = "view.zoom_reset";
pub const MENU_SHOW: &str = "app.show";
pub const MENU_HIDE: &str = "app.hide";
pub const MENU_RELOAD_GROK: &str = "app.reload_grok";
pub const MENU_OPEN_SETTINGS: &str = "app.open_settings";
pub const MENU_QUIT: &str = "app.quit";
pub const MENU_SCRATCHPAD_TOGGLE: &str = "scratchpad.toggle";
pub const MENU_SCRATCHPAD_IMPORT: &str = "scratchpad.import";
pub const MENU_SCRATCHPAD_EXPORT: &str = "scratchpad.export";
pub const MENU_SCRATCHPAD_CLEAR: &str = "scratchpad.clear";
pub const MENU_PROMPT_LIBRARY_TOGGLE: &str = "prompt_library.toggle";
pub const MENU_PROMPT_LIBRARY_IMPORT: &str = "prompt_library.import";
pub const MENU_PROMPT_LIBRARY_EXPORT: &str = "prompt_library.export";
pub const MENU_COMMAND_PALETTE: &str = "shell.command_palette";
pub const MENU_WORKSPACE_SWITCH: &str = "workspace.switch";
pub const MENU_WORKSPACE_CREATE: &str = "workspace.create";

pub fn install_app_menu(app: &AppHandle) -> tauri::Result<()> {
    let app_menu = Submenu::with_items(
        app,
        setup::APP_TITLE,
        true,
        &[
            &PredefinedMenuItem::about(app, None, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, MENU_SCRATCHPAD_IMPORT, "Import Scratchpad...", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_SCRATCHPAD_EXPORT, "Export Scratchpad...", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_SCRATCHPAD_CLEAR, "Clear Scratchpad", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_PROMPT_LIBRARY_IMPORT, "Import Prompt Library...", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_PROMPT_LIBRARY_EXPORT, "Export Prompt Library...", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, MENU_ZOOM_IN, "Zoom In", true, Some("CmdOrCtrl+="))?,
            &MenuItem::with_id(app, MENU_ZOOM_OUT, "Zoom Out", true, Some("CmdOrCtrl+-"))?,
            &MenuItem::with_id(app, MENU_ZOOM_RESET, "Actual Size", true, Some("CmdOrCtrl+0"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_SCRATCHPAD_TOGGLE, "Toggle Scratchpad", true, Some("CmdOrCtrl+Shift+K"))?,
            &MenuItem::with_id(app, MENU_PROMPT_LIBRARY_TOGGLE, "Toggle Prompt Library", true, Some("CmdOrCtrl+Shift+P"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, MENU_COMMAND_PALETTE, "Command Palette", true, Some("CmdOrCtrl+K"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
        ],
    )?;

    let workspace_menu = Submenu::with_items(
        app,
        "Workspaces",
        true,
        &[
            &MenuItem::with_id(app, MENU_WORKSPACE_SWITCH, "Switch Workspace...", true, None::<&str>)?,
            &MenuItem::with_id(app, MENU_WORKSPACE_CREATE, "Create Workspace...", true, None::<&str>)?,
        ],
    )?;

    let shell_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;

    let help_menu = Submenu::with_items(app, "Help", true, &[])?;

    Menu::with_items(
        app,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &workspace_menu,
            &shell_menu,
            &help_menu,
        ],
    )?
    .set_as_app_menu()?;

    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, event_id: &str) {
    let store = app.state::<crate::persistence::state::PersistedStateStore>();

    match event_id {
        MENU_SHOW => {
            let _ = setup::show_main_window(app);
        }
        MENU_HIDE => {
            let _ = setup::hide_main_window(app);
        }
        MENU_RELOAD_GROK => {
            let runtime_state = app.state::<grok::GrokRuntimeState>();
            let _ = grok::reload(app, &runtime_state);
        }
        MENU_OPEN_SETTINGS => {
            let _ = setup::show_main_window(app);
        }
        MENU_SCRATCHPAD_TOGGLE => {
            let _ = scratchpad::toggle_scratchpad_state(app.clone(), store.clone());
        }
        MENU_SCRATCHPAD_IMPORT => {
            let _ = scratchpad::import_scratchpad_content(app.clone(), store.clone());
        }
        MENU_SCRATCHPAD_EXPORT => {
            let _ = scratchpad::export_scratchpad_content(app.clone(), store.clone(), None);
        }
        MENU_SCRATCHPAD_CLEAR => {
            let _ = scratchpad::clear_scratchpad_content(app.clone(), store.clone());
        }
        MENU_PROMPT_LIBRARY_TOGGLE => {
            let _ = prompt_library::toggle_prompt_library_state(app.clone(), store.clone());
        }
        MENU_PROMPT_LIBRARY_IMPORT => {
            let _ = prompt_library::import_prompt_library(app.clone(), store.clone());
        }
        MENU_PROMPT_LIBRARY_EXPORT => {
            let _ = prompt_library::export_prompt_library(app.clone(), store.clone());
        }
        MENU_COMMAND_PALETTE => {
            crate::window::events::open_command_palette(app);
        }
        MENU_WORKSPACE_SWITCH => {
            crate::window::events::open_command_palette_with_query(app, "switch workspace");
        }
        MENU_WORKSPACE_CREATE => {
            crate::window::events::open_command_palette_with_query(app, "create workspace");
        }
        MENU_QUIT => {
            app.exit(0);
        }
        MENU_ZOOM_IN => {
            let _ = zoom::zoom_in(app, &store);
        }
        MENU_ZOOM_OUT => {
            let _ = zoom::zoom_out(app, &store);
        }
        MENU_ZOOM_RESET => {
            let _ = zoom::zoom_reset(app, &store);
        }
        _ => {}
    }
}
