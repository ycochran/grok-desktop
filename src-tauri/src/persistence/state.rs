use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub theme: String,
    pub close_behavior: String,
    pub restore_window_state: bool,
    pub restore_zoom: bool,
    pub zoom_level: f64,
    pub open_external_links_in_browser: bool,
    pub navigation_debug_logging: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: String::from("system"),
            close_behavior: String::from("quit"),
            restore_window_state: true,
            restore_zoom: true,
            zoom_level: 1.0,
            open_external_links_in_browser: true,
            navigation_debug_logging: false,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        self.close_behavior = match self.close_behavior.as_str() {
            "hide" => String::from("hide"),
            "close" | "quit" => String::from("quit"),
            _ => Self::default().close_behavior,
        };
        self.zoom_level = clamp_zoom(self.zoom_level);
        self
    }

    pub fn navigation_debug_logging_enabled(&self) -> bool {
        self.navigation_debug_logging
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    pub x: f64,
    pub y: f64,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1440.0,
            height: 920.0,
            x: 0.0,
            y: 0.0,
            maximized: false,
        }
    }
}

impl WindowState {
    pub fn normalized(self) -> Self {
        let default = Self::default();

        let width = normalize_dimension(self.width, 960.0, 6000.0, default.width);
        let height = normalize_dimension(self.height, 640.0, 4000.0, default.height);
        let x = normalize_coordinate(self.x, default.x);
        let y = normalize_coordinate(self.y, default.y);

        Self {
            width,
            height,
            x,
            y,
            maximized: self.maximized,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScratchpadState {
    pub content: String,
    pub is_open: bool,
}

impl Default for ScratchpadState {
    fn default() -> Self {
        Self {
            content: String::new(),
            is_open: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptSnippet {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PromptLibraryState {
    pub is_open: bool,
    pub prompts: Vec<PromptSnippet>,
}

impl Default for PromptLibraryState {
    fn default() -> Self {
        Self {
            is_open: true,
            prompts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub last_used_at: u64,
}

impl Workspace {
    pub fn default_workspace() -> Self {
        Self {
            id: String::from("default"),
            name: String::from("Default"),
            is_default: true,
            last_used_at: current_timestamp(),
        }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::default_workspace()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspaceShellState {
    pub workspace_id: String,
    pub scratchpad: ScratchpadState,
    pub prompt_library_is_open: bool,
}

impl WorkspaceShellState {
    pub(crate) fn new(workspace_id: &str) -> Self {
        Self {
            workspace_id: workspace_id.to_string(),
            scratchpad: ScratchpadState::default(),
            prompt_library_is_open: PromptLibraryState::default().is_open,
        }
    }
}

impl Default for WorkspaceShellState {
    fn default() -> Self {
        Self::new("default")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspacesState {
    pub active_workspace_id: String,
    pub workspaces: Vec<Workspace>,
    pub shell_states: Vec<WorkspaceShellState>,
}

impl Default for WorkspacesState {
    fn default() -> Self {
        let workspace = Workspace::default_workspace();
        Self {
            active_workspace_id: workspace.id.clone(),
            workspaces: vec![workspace.clone()],
            shell_states: vec![WorkspaceShellState::new(&workspace.id)],
        }
    }
}

impl WorkspacesState {
    pub fn normalized(
        mut self,
        legacy_scratchpad: &ScratchpadState,
        legacy_prompt_library: &PromptLibraryState,
    ) -> Self {
        let had_shell_states = !self.shell_states.is_empty();

        if self.workspaces.is_empty() {
            self.workspaces.push(Workspace::default_workspace());
        }

        let mut default_index = self.workspaces.iter().position(|workspace| workspace.is_default);
        if default_index.is_none() {
            default_index = self
                .workspaces
                .iter()
                .position(|workspace| workspace.id == "default");
        }

        let default_index = default_index.unwrap_or(0);
        for (index, workspace) in self.workspaces.iter_mut().enumerate() {
            workspace.is_default = index == default_index;
            if workspace.id.trim().is_empty() {
                workspace.id = if workspace.is_default {
                    String::from("default")
                } else {
                    format!("workspace-{}", index + 1)
                };
            }
            if workspace.name.trim().is_empty() {
                workspace.name = if workspace.is_default {
                    String::from("Default")
                } else {
                    String::from("Workspace")
                };
            }
        }

        if !self
            .workspaces
            .iter()
            .any(|workspace| workspace.id == self.active_workspace_id)
        {
            self.active_workspace_id = self.workspaces[default_index].id.clone();
        }

        self.shell_states
            .retain(|state| self.workspaces.iter().any(|workspace| workspace.id == state.workspace_id));

        for workspace in &self.workspaces {
            if !self
                .shell_states
                .iter()
                .any(|state| state.workspace_id == workspace.id)
            {
                self.shell_states.push(WorkspaceShellState::new(&workspace.id));
            }
        }

        if !had_shell_states {
            if let Some(shell_state) = self.active_shell_state_mut() {
                shell_state.scratchpad = legacy_scratchpad.clone();
                shell_state.prompt_library_is_open = legacy_prompt_library.is_open;
            }
        }

        self
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.id == self.active_workspace_id)
    }

    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        let active_workspace_id = self.active_workspace_id.clone();
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == active_workspace_id)
    }

    pub fn active_shell_state(&self) -> Option<&WorkspaceShellState> {
        self.shell_states
            .iter()
            .find(|state| state.workspace_id == self.active_workspace_id)
    }

    pub fn active_shell_state_mut(&mut self) -> Option<&mut WorkspaceShellState> {
        let active_workspace_id = self.active_workspace_id.clone();
        self.shell_states
            .iter_mut()
            .find(|state| state.workspace_id == active_workspace_id)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistedState {
    pub settings: AppSettings,
    pub window: WindowState,
    pub scratchpad: ScratchpadState,
    pub prompt_library: PromptLibraryState,
    pub workspaces: WorkspacesState,
}

impl PersistedState {
    pub fn normalized(mut self) -> Self {
        self.settings = self.settings.normalized();
        self.window = self.window.normalized();
        self.sync_active_workspace_shell_state();
        self.workspaces = self
            .workspaces
            .normalized(&self.scratchpad, &self.prompt_library);

        if let Some(active_shell_state) = self.workspaces.active_shell_state() {
            self.scratchpad = active_shell_state.scratchpad.clone();
            self.prompt_library.is_open = active_shell_state.prompt_library_is_open;
        }

        self
    }

    pub fn sync_active_workspace_shell_state(&mut self) {
        if let Some(active_shell_state) = self.workspaces.active_shell_state_mut() {
            active_shell_state.scratchpad = self.scratchpad.clone();
            active_shell_state.prompt_library_is_open = self.prompt_library.is_open;
        }
    }
}

pub struct PersistedStateStore {
    pub inner: Mutex<PersistedState>,
}

impl PersistedStateStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(PersistedState::default()),
        }
    }
}

impl PersistedState {
    pub fn load(app: &AppHandle) -> tauri::Result<Self> {
        let path = state_file_path(app)?;
        if !path.exists() {
            return Ok(Self::default().normalized());
        }

        let contents = fs::read_to_string(path)?;
        let state = serde_json::from_str::<Self>(&contents).unwrap_or_default();
        Ok(state.normalized())
    }

    pub fn save(&self, app: &AppHandle) -> tauri::Result<()> {
        let path = state_file_path(app)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.clone().normalized())?;
        fs::write(path, json)?;
        Ok(())
    }
}

pub fn clamp_zoom(zoom_level: f64) -> f64 {
    zoom_level.clamp(0.75, 2.0)
}

#[allow(dead_code)]
pub fn persisted_state_exists(app: &AppHandle) -> bool {
    state_file_path(app)
        .map(|path| path.exists())
        .unwrap_or(false)
}

fn state_file_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    let mut path = app.path().app_config_dir()?;
    path.push("state.json");
    Ok(path)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn normalize_dimension(value: f64, min: f64, max: f64, fallback: f64) -> f64 {
    if !value.is_finite() {
        return fallback;
    }

    value.clamp(min, max)
}

fn normalize_coordinate(value: f64, fallback: f64) -> f64 {
    if !value.is_finite() {
        return fallback;
    }

    value.clamp(-20_000.0, 20_000.0)
}
