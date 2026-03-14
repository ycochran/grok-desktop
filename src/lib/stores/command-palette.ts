import { writable } from "svelte/store";

const RECENT_COMMANDS_KEY = "grok-desktop.command-palette.recent";
const RECENT_COMMAND_LIMIT = 8;

const { subscribe, set, update } = writable(false);
const recentStore = writable<string[]>(loadRecentCommands());

export const commandPalette = {
  subscribe,
};

export const recentCommandIds = {
  subscribe: recentStore.subscribe,
};

export function openCommandPalette(): void {
  set(true);
}

export function closeCommandPalette(): void {
  set(false);
}

export function toggleCommandPalette(): void {
  update((current) => !current);
}

export function recordRecentCommand(commandId: string): void {
  recentStore.update((current) => {
    const next = [commandId, ...current.filter((id) => id !== commandId)].slice(0, RECENT_COMMAND_LIMIT);
    saveRecentCommands(next);
    return next;
  });
}

function loadRecentCommands(): string[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const raw = window.localStorage.getItem(RECENT_COMMANDS_KEY);
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw);
    return Array.isArray(parsed)
      ? parsed.filter((value): value is string => typeof value === "string").slice(0, RECENT_COMMAND_LIMIT)
      : [];
  } catch {
    return [];
  }
}

function saveRecentCommands(commandIds: string[]): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(RECENT_COMMANDS_KEY, JSON.stringify(commandIds));
  } catch {
    // Keep recent-command tracking best-effort only.
  }
}
