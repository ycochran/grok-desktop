use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::commands::{prompt_library, scratchpad};
use crate::persistence::state::{
    PersistedStateStore, Workspace, WorkspaceShellState, WorkspacesState,
};

pub const WORKSPACE_STATE_EVENT: &str = "app://workspace-state";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStateSnapshot {
    pub active_workspace_id: String,
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceActionResult {
    pub state: WorkspaceStateSnapshot,
    pub message: String,
}

#[tauri::command]
pub fn get_workspace_state(
    store: State<'_, PersistedStateStore>,
) -> Result<WorkspaceStateSnapshot, String> {
    let guard = store
        .inner
        .lock()
        .map_err(|_| String::from("settings store mutex poisoned"))?;

    Ok(snapshot(&guard.workspaces))
}

#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    name: Option<String>,
) -> Result<WorkspaceActionResult, String> {
    let trimmed_name = sanitize_workspace_name(name.unwrap_or_else(|| String::from("New Workspace")));
    let workspace_id = create_workspace_id(&trimmed_name);

    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        guard.sync_active_workspace_shell_state();
        guard.workspaces.workspaces.push(Workspace {
            id: workspace_id.clone(),
            name: trimmed_name.clone(),
            is_default: false,
            last_used_at: current_timestamp(),
        });
        guard
            .workspaces
            .shell_states
            .push(WorkspaceShellState::new(&workspace_id));
        guard.workspaces.active_workspace_id = workspace_id;
        if let Some(active_workspace) = guard.workspaces.active_workspace_mut() {
            active_workspace.last_used_at = current_timestamp();
        }
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_workspace_updates(&app, &updated_state);

    Ok(WorkspaceActionResult {
        state: snapshot(&updated_state.workspaces),
        message: format!("Workspace \"{}\" created.", trimmed_name),
    })
}

#[tauri::command]
pub fn rename_workspace(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    workspace_id: String,
    name: String,
) -> Result<WorkspaceActionResult, String> {
    let trimmed_name = sanitize_workspace_name(name);

    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;
        let Some(workspace) = guard
            .workspaces
            .workspaces
            .iter_mut()
            .find(|workspace| workspace.id == workspace_id)
        else {
            return Err(String::from("Workspace not found."));
        };

        workspace.name = trimmed_name.clone();
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_workspace_updates(&app, &updated_state);

    Ok(WorkspaceActionResult {
        state: snapshot(&updated_state.workspaces),
        message: format!("Workspace renamed to \"{}\".", trimmed_name),
    })
}

#[tauri::command]
pub fn switch_workspace(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    workspace_id: String,
) -> Result<WorkspaceActionResult, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;

        if !guard
            .workspaces
            .workspaces
            .iter()
            .any(|workspace| workspace.id == workspace_id)
        {
            return Err(String::from("Workspace not found."));
        }

        guard.sync_active_workspace_shell_state();
        guard.workspaces.active_workspace_id = workspace_id.clone();
        if let Some(active_workspace) = guard.workspaces.active_workspace_mut() {
            active_workspace.last_used_at = current_timestamp();
        }
        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_workspace_updates(&app, &updated_state);

    let active_name = updated_state
        .workspaces
        .active_workspace()
        .map(|workspace| workspace.name.clone())
        .unwrap_or_else(|| String::from("Workspace"));

    Ok(WorkspaceActionResult {
        state: snapshot(&updated_state.workspaces),
        message: format!("Switched to \"{}\".", active_name),
    })
}

#[tauri::command]
pub fn delete_workspace(
    app: AppHandle,
    store: State<'_, PersistedStateStore>,
    workspace_id: String,
) -> Result<WorkspaceActionResult, String> {
    let updated_state = {
        let mut guard = store
            .inner
            .lock()
            .map_err(|_| String::from("settings store mutex poisoned"))?;

        let Some(workspace) = guard
            .workspaces
            .workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
        else {
            return Err(String::from("Workspace not found."));
        };

        if workspace.is_default {
            return Err(String::from("The default workspace cannot be deleted."));
        }

        guard.sync_active_workspace_shell_state();

        guard
            .workspaces
            .workspaces
            .retain(|workspace| workspace.id != workspace_id);
        guard
            .workspaces
            .shell_states
            .retain(|state| state.workspace_id != workspace_id);

        if guard.workspaces.active_workspace_id == workspace_id {
            let fallback_id = guard
                .workspaces
                .workspaces
                .iter()
                .find(|workspace| workspace.is_default)
                .map(|workspace| workspace.id.clone())
                .or_else(|| guard.workspaces.workspaces.first().map(|workspace| workspace.id.clone()))
                .unwrap_or_else(|| String::from("default"));
            guard.workspaces.active_workspace_id = fallback_id;
        }

        if let Some(active_workspace) = guard.workspaces.active_workspace_mut() {
            active_workspace.last_used_at = current_timestamp();
        }

        guard.clone().normalized()
    };

    updated_state.save(&app).map_err(|error| error.to_string())?;
    emit_workspace_updates(&app, &updated_state);

    Ok(WorkspaceActionResult {
        state: snapshot(&updated_state.workspaces),
        message: String::from("Workspace deleted."),
    })
}

fn emit_workspace_updates(app: &AppHandle, state: &crate::persistence::state::PersistedState) {
    let _ = app.emit_to("main", WORKSPACE_STATE_EVENT, snapshot(&state.workspaces));
    scratchpad::emit_scratchpad_state(app, &state.scratchpad);
    prompt_library::emit_prompt_library_state(app, &state.prompt_library);
}

fn snapshot(workspaces: &WorkspacesState) -> WorkspaceStateSnapshot {
    let mut items = workspaces.workspaces.clone();
    items.sort_by(|left, right| {
        right
            .last_used_at
            .cmp(&left.last_used_at)
            .then_with(|| left.name.cmp(&right.name))
    });

    WorkspaceStateSnapshot {
        active_workspace_id: workspaces.active_workspace_id.clone(),
        workspaces: items,
    }
}

fn sanitize_workspace_name(name: String) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return String::from("Workspace");
    }

    trimmed.chars().take(48).collect()
}

fn create_workspace_id(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    format!("ws-{}-{}", if slug.is_empty() { "workspace" } else { &slug }, current_timestamp())
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
