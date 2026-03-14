import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";

export interface ScratchpadState {
  content: string;
  isOpen: boolean;
}

interface ScratchpadActionResult {
  state: ScratchpadState;
  message: string;
}

const SCRATCHPAD_STATE_EVENT = "app://scratchpad-state";

const defaultScratchpadState: ScratchpadState = {
  content: "",
  isOpen: true,
};

const { subscribe, set, update } = writable<ScratchpadState>(defaultScratchpadState);

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let initialized = false;
let unlistenScratchpadState: UnlistenFn | null = null;

export const scratchpad = {
  subscribe,
};

export async function initializeScratchpad(): Promise<void> {
  if (initialized) {
    return;
  }

  initialized = true;

  try {
    const persisted = await invoke<ScratchpadState>("get_scratchpad_state");
    set(normalizeScratchpadState(persisted));
    unlistenScratchpadState = await listen<ScratchpadState>(SCRATCHPAD_STATE_EVENT, (event) => {
      set(normalizeScratchpadState(event.payload));
    });
  } catch (error) {
    console.error("Failed to restore scratchpad state, using defaults.", error);
    set(defaultScratchpadState);
  }
}

export function setScratchpadContent(content: string): void {
  update((current) => {
    const next = normalizeScratchpadState({ ...current, content });
    queuePersist(next);
    return next;
  });
}

export function setScratchpadOpen(isOpen: boolean): void {
  update((current) => {
    const next = normalizeScratchpadState({ ...current, isOpen });
    queuePersist(next);
    return next;
  });
}

export function clearScratchpad(): void {
  update((current) => {
    const next = normalizeScratchpadState({ ...current, content: "" });
    queuePersist(next);
    return next;
  });
}

export function appendToScratchpad(content: string): void {
  update((current) => {
    const separator = current.content.trim() ? "\n\n" : "";
    const next = normalizeScratchpadState({
      ...current,
      isOpen: true,
      content: `${current.content}${separator}${content}`,
    });
    queuePersist(next);
    return next;
  });
}

export async function copyScratchpadContent(content: string): Promise<void> {
  await invoke("copy_scratchpad_content", { content });
}

export async function clearScratchpadRemotely(): Promise<void> {
  const persisted = await invoke<ScratchpadState>("clear_scratchpad_content");
  set(normalizeScratchpadState(persisted));
}

export async function toggleScratchpad(): Promise<void> {
  const persisted = await invoke<ScratchpadState>("toggle_scratchpad_state");
  set(normalizeScratchpadState(persisted));
}

export async function importScratchpad(): Promise<string> {
  const result = await invoke<ScratchpadActionResult>("import_scratchpad_content");
  set(normalizeScratchpadState(result.state));
  return result.message;
}

export async function exportScratchpad(content: string): Promise<string> {
  const result = await invoke<ScratchpadActionResult>("export_scratchpad_content", {
    content,
  });
  return result.message;
}

function queuePersist(state: ScratchpadState): void {
  if (saveTimer) {
    clearTimeout(saveTimer);
  }

  saveTimer = setTimeout(() => {
    void persistScratchpadState(state);
  }, 300);
}

async function persistScratchpadState(state: ScratchpadState): Promise<void> {
  try {
    const persisted = await invoke<ScratchpadState>("update_scratchpad_state", {
      scratchpad: state,
    });
    set(normalizeScratchpadState(persisted));
  } catch (error) {
    console.error("Failed to persist scratchpad state.", error);
  }
}

function normalizeScratchpadState(state: ScratchpadState): ScratchpadState {
  return {
    ...defaultScratchpadState,
    ...state,
    content: typeof state.content === "string" ? state.content : "",
    isOpen: state.isOpen ?? defaultScratchpadState.isOpen,
  };
}
