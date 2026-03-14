import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export type ThemeSetting = "system" | "light" | "dark";
export type CloseBehavior = "quit" | "hide";

export interface AppSettings {
  theme: ThemeSetting;
  closeBehavior: CloseBehavior;
  restoreWindowState: boolean;
  restoreZoom: boolean;
  zoomLevel: number;
  openExternalLinksInBrowser: boolean;
  navigationDebugLogging: boolean;
}

const defaultSettings: AppSettings = {
  theme: "system",
  closeBehavior: "quit",
  restoreWindowState: true,
  restoreZoom: true,
  zoomLevel: 1,
  openExternalLinksInBrowser: true,
  navigationDebugLogging: false,
};

const { subscribe, set, update } = writable<AppSettings>(defaultSettings);

export const appSettings = {
  subscribe,
};

export async function initializeAppSettings(): Promise<void> {
  try {
    const settings = await invoke<AppSettings>("get_settings");
    replaceAppSettings(settings);
  } catch (error) {
    console.error("Failed to load app settings, using defaults.", error);
    set(defaultSettings);
  }
}

export async function patchAppSettings(partial: Partial<AppSettings>): Promise<void> {
  let nextSettings = defaultSettings;

  update((current) => {
    nextSettings = normalizeSettings({ ...current, ...partial });
    return nextSettings;
  });

  try {
    const persisted = await invoke<AppSettings>("update_settings", { settings: nextSettings });
    replaceAppSettings(persisted);
  } catch (error) {
    console.error("Failed to persist app settings.", error);
    replaceAppSettings(nextSettings);
  }
}

export function replaceAppSettings(settings: AppSettings): void {
  set(normalizeSettings(settings));
}

export function updateAppSettingsLocally(partial: Partial<AppSettings>): void {
  update((current) => normalizeSettings({ ...current, ...partial }));
}

function normalizeSettings(settings: AppSettings): AppSettings {
  return {
    ...defaultSettings,
    ...settings,
    closeBehavior: normalizeCloseBehavior(settings.closeBehavior),
    zoomLevel: clampZoom(settings.zoomLevel ?? defaultSettings.zoomLevel),
  };
}

function normalizeCloseBehavior(closeBehavior: string | undefined): CloseBehavior {
  if (closeBehavior === "hide") {
    return "hide";
  }

  // Migrate the Milestone 1 scaffold value to the native close semantics used now.
  if (closeBehavior === "close" || closeBehavior === "quit") {
    return "quit";
  }

  return defaultSettings.closeBehavior;
}

export function clampZoom(zoomLevel: number): number {
  return Math.min(2, Math.max(0.75, Number.isFinite(zoomLevel) ? zoomLevel : 1));
}
