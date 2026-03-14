import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { get, writable } from "svelte/store";
import { appendToScratchpad, setScratchpadOpen } from "$lib/stores/scratchpad";

export interface PromptSnippet {
  id: string;
  title: string;
  content: string;
  tags: string[];
  isFavorite: boolean;
}

export interface PromptLibraryState {
  isOpen: boolean;
  prompts: PromptSnippet[];
}

interface PromptLibraryActionResult {
  state: PromptLibraryState;
  message: string;
}

const PROMPT_LIBRARY_STATE_EVENT = "app://prompt-library-state";

const defaultPromptLibraryState: PromptLibraryState = {
  isOpen: true,
  prompts: [],
};

const { subscribe, set, update } = writable<PromptLibraryState>(defaultPromptLibraryState);

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let initialized = false;
let unlistenPromptLibraryState: UnlistenFn | null = null;

export const promptLibrary = {
  subscribe,
};

export async function initializePromptLibrary(): Promise<void> {
  if (initialized) {
    return;
  }

  initialized = true;

  try {
    const persisted = await invoke<PromptLibraryState>("get_prompt_library_state");
    set(normalizePromptLibraryState(persisted));
    unlistenPromptLibraryState = await listen<PromptLibraryState>(PROMPT_LIBRARY_STATE_EVENT, (event) => {
      set(normalizePromptLibraryState(event.payload));
    });
  } catch (error) {
    console.error("Failed to restore prompt library state, using defaults.", error);
    set(defaultPromptLibraryState);
  }
}

export function setPromptLibraryOpen(isOpen: boolean): void {
  update((current) => {
    const next = normalizePromptLibraryState({ ...current, isOpen });
    queuePersist(next);
    return next;
  });
}

export async function togglePromptLibrary(): Promise<string> {
  const result = await invoke<PromptLibraryActionResult>("toggle_prompt_library_state");
  set(normalizePromptLibraryState(result.state));
  return result.message;
}

export function createPrompt(): string {
  const prompt = {
    id: crypto.randomUUID?.() ?? `prompt-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    title: "Untitled prompt",
    content: "",
    tags: [],
    isFavorite: false,
  };

  update((current) => {
    const next = normalizePromptLibraryState({
      ...current,
      isOpen: true,
      prompts: [prompt, ...current.prompts],
    });
    queuePersist(next);
    return next;
  });

  return prompt.id;
}

export function duplicatePrompt(id: string): string | null {
  const sourcePrompt = get(promptLibrary).prompts.find((prompt) => prompt.id === id);
  if (!sourcePrompt) {
    return null;
  }

  const duplicate = {
    ...sourcePrompt,
    id: crypto.randomUUID?.() ?? `prompt-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    title: `${sourcePrompt.title} Copy`,
  };

  update((current) => {
    const next = normalizePromptLibraryState({
      ...current,
      isOpen: true,
      prompts: [duplicate, ...current.prompts],
    });
    queuePersist(next);
    return next;
  });

  return duplicate.id;
}

export function updatePrompt(id: string, partial: Partial<PromptSnippet>): void {
  update((current) => {
    const next = normalizePromptLibraryState({
      ...current,
      prompts: current.prompts.map((prompt) =>
        prompt.id === id
          ? {
              ...prompt,
              ...partial,
            }
          : prompt,
      ),
    });
    queuePersist(next);
    return next;
  });
}

export function deletePrompt(id: string): void {
  update((current) => {
    const next = normalizePromptLibraryState({
      ...current,
      prompts: current.prompts.filter((prompt) => prompt.id !== id),
    });
    queuePersist(next);
    return next;
  });
}

export async function copyPromptContent(content: string): Promise<void> {
  await invoke("copy_scratchpad_content", { content });
}

export async function importPromptLibrary(): Promise<string> {
  const result = await invoke<PromptLibraryActionResult>("import_prompt_library");
  set(normalizePromptLibraryState(result.state));
  return result.message;
}

export async function exportPromptLibrary(): Promise<string> {
  const result = await invoke<PromptLibraryActionResult>("export_prompt_library");
  return result.message;
}

export function appendPromptToScratchpad(content: string): void {
  if (!content.trim()) {
    return;
  }

  setScratchpadOpen(true);
  appendToScratchpad(content);
}

export function getPromptById(id: string | null): PromptSnippet | undefined {
  return get(promptLibrary).prompts.find((prompt) => prompt.id === id);
}

function queuePersist(state: PromptLibraryState): void {
  if (saveTimer) {
    clearTimeout(saveTimer);
  }

  saveTimer = setTimeout(() => {
    void persistPromptLibraryState(state);
  }, 300);
}

async function persistPromptLibraryState(state: PromptLibraryState): Promise<void> {
  try {
    const persisted = await invoke<PromptLibraryState>("update_prompt_library_state", {
      promptLibrary: state,
    });
    set(normalizePromptLibraryState(persisted));
  } catch (error) {
    console.error("Failed to persist prompt library state.", error);
  }
}

function normalizePromptLibraryState(state: PromptLibraryState): PromptLibraryState {
  return {
    ...defaultPromptLibraryState,
    ...state,
    isOpen: state.isOpen ?? defaultPromptLibraryState.isOpen,
    prompts: Array.isArray(state.prompts)
      ? state.prompts.map((prompt) => ({
          id: typeof prompt.id === "string" ? prompt.id : "",
          title: typeof prompt.title === "string" && prompt.title.trim() ? prompt.title : "Untitled prompt",
          content: typeof prompt.content === "string" ? prompt.content : "",
          tags: Array.isArray(prompt.tags)
            ? prompt.tags
                .map((tag) => (typeof tag === "string" ? tag.trim() : ""))
                .filter(Boolean)
            : [],
          isFavorite: Boolean(prompt.isFavorite),
        }))
      : [],
  };
}
