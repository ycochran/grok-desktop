import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { derived, writable } from "svelte/store";

export interface Workspace {
  id: string;
  name: string;
  isDefault: boolean;
  lastUsedAt: number;
}

export interface WorkspaceState {
  activeWorkspaceId: string;
  workspaces: Workspace[];
}

interface WorkspaceActionResult {
  state: WorkspaceState;
  message: string;
}

const WORKSPACE_STATE_EVENT = "app://workspace-state";

const defaultWorkspaceState: WorkspaceState = {
  activeWorkspaceId: "default",
  workspaces: [
    {
      id: "default",
      name: "Default",
      isDefault: true,
      lastUsedAt: 0,
    },
  ],
};

const { subscribe, set } = writable<WorkspaceState>(defaultWorkspaceState);

let initialized = false;
let unlistenWorkspaceState: UnlistenFn | null = null;

export const workspaceState = {
  subscribe,
};

export const activeWorkspace = derived(workspaceState, ($workspaceState) => {
  return (
    $workspaceState.workspaces.find((workspace) => workspace.id === $workspaceState.activeWorkspaceId) ??
    $workspaceState.workspaces[0] ??
    null
  );
});

export async function initializeWorkspaceState(): Promise<void> {
  if (initialized) {
    return;
  }

  initialized = true;

  try {
    const persisted = await invoke<WorkspaceState>("get_workspace_state");
    set(normalizeWorkspaceState(persisted));
    unlistenWorkspaceState = await listen<WorkspaceState>(WORKSPACE_STATE_EVENT, (event) => {
      set(normalizeWorkspaceState(event.payload));
    });
  } catch (error) {
    console.error("Failed to restore workspace state, using defaults.", error);
    set(defaultWorkspaceState);
  }
}

export async function createWorkspace(name: string): Promise<string> {
  const result = await invoke<WorkspaceActionResult>("create_workspace", {
    name,
  });
  set(normalizeWorkspaceState(result.state));
  return result.message;
}

export async function renameWorkspace(workspaceId: string, name: string): Promise<string> {
  const result = await invoke<WorkspaceActionResult>("rename_workspace", {
    workspaceId,
    name,
  });
  set(normalizeWorkspaceState(result.state));
  return result.message;
}

export async function switchWorkspace(workspaceId: string): Promise<string> {
  const result = await invoke<WorkspaceActionResult>("switch_workspace", {
    workspaceId,
  });
  set(normalizeWorkspaceState(result.state));
  return result.message;
}

export async function deleteWorkspace(workspaceId: string): Promise<string> {
  const result = await invoke<WorkspaceActionResult>("delete_workspace", {
    workspaceId,
  });
  set(normalizeWorkspaceState(result.state));
  return result.message;
}

export function disposeWorkspaceState(): void {
  unlistenWorkspaceState?.();
  unlistenWorkspaceState = null;
  initialized = false;
}

function normalizeWorkspaceState(state: WorkspaceState): WorkspaceState {
  const workspaces = Array.isArray(state.workspaces)
    ? state.workspaces.map((workspace) => ({
        id: typeof workspace.id === "string" && workspace.id.trim() ? workspace.id : "default",
        name: typeof workspace.name === "string" && workspace.name.trim() ? workspace.name.trim() : "Workspace",
        isDefault: Boolean(workspace.isDefault),
        lastUsedAt: Number.isFinite(workspace.lastUsedAt) ? workspace.lastUsedAt : 0,
      }))
    : defaultWorkspaceState.workspaces;

  const activeWorkspaceId =
    typeof state.activeWorkspaceId === "string" &&
    workspaces.some((workspace) => workspace.id === state.activeWorkspaceId)
      ? state.activeWorkspaceId
      : (workspaces.find((workspace) => workspace.isDefault)?.id ?? workspaces[0]?.id ?? "default");

  return {
    activeWorkspaceId,
    workspaces,
  };
}
