import { writable } from "svelte/store";

const SIDEBAR_STATE_KEY = "grok-desktop.sidebar-collapsed";

const { subscribe, set, update } = writable(loadInitialValue());

export const sidebarCollapsed = {
  subscribe,
};

export function setSidebarCollapsed(collapsed: boolean): void {
  set(collapsed);
  persist(collapsed);
}

export function toggleSidebarCollapsed(): void {
  update((current) => {
    const next = !current;
    persist(next);
    return next;
  });
}

function loadInitialValue(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  try {
    return window.localStorage.getItem(SIDEBAR_STATE_KEY) === "true";
  } catch {
    return false;
  }
}

function persist(collapsed: boolean): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(SIDEBAR_STATE_KEY, String(collapsed));
  } catch {
    // Keep sidebar persistence best-effort only.
  }
}
