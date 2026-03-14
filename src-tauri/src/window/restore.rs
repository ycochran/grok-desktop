use tauri::WebviewWindow;

use crate::persistence::state::PersistedStateStore;

pub fn restore_main_window(window: &WebviewWindow, _store: &PersistedStateStore) -> tauri::Result<()> {
    // The tauri-plugin-window-state plugin handles persisting and restoring
    // window size and position automatically. We only need to ensure the
    // window opens on the active display (where the cursor is) when it would
    // otherwise appear on a different monitor.
    reposition_to_active_display(window)?;

    window.show()?;
    window.set_focus()?;
    Ok(())
}

/// If the window's current position is not on the monitor where the cursor
/// is, center the window on the cursor's monitor (preserving its size).
fn reposition_to_active_display(window: &WebviewWindow) -> tauri::Result<()> {
    let cursor = match window.cursor_position() {
        Ok(pos) => pos,
        Err(_) => return Ok(()), // Can't determine cursor; leave window where the plugin put it.
    };

    let cursor_monitor = match window.monitor_from_point(cursor.x, cursor.y)? {
        Some(m) => m,
        None => return Ok(()),
    };

    // Check if the window is already on the cursor's monitor.
    if let Some(current_monitor) = window.current_monitor()? {
        let ca = current_monitor.work_area();
        let cb = cursor_monitor.work_area();
        if ca.position.x == cb.position.x && ca.position.y == cb.position.y {
            return Ok(()); // Already on the active display.
        }
    }

    // Center the window on the cursor's monitor, keeping its current size.
    let size = window.outer_size()?;
    let work = cursor_monitor.work_area();
    let x = work.position.x + ((work.size.width as i32 - size.width as i32) / 2);
    let y = work.position.y + ((work.size.height as i32 - size.height as i32) / 2);

    window.set_position(tauri::PhysicalPosition::new(x, y))?;
    Ok(())
}
