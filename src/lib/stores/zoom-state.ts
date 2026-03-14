import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { derived } from "svelte/store";
import { appSettings, clampZoom, updateAppSettingsLocally } from "$lib/stores/app-settings";

const ZOOM_CHANGED_EVENT = "app://zoom-changed";

interface ZoomChangedPayload {
  zoomLevel: number;
}

let initialized = false;
let unlistenZoomChanged: UnlistenFn | null = null;

export const zoomLevel = derived(appSettings, ($appSettings) => clampZoom($appSettings.zoomLevel));
export const zoomLevelLabel = derived(zoomLevel, ($zoomLevel) => `${Math.round($zoomLevel * 100)}%`);

export async function initializeZoomState(): Promise<void> {
  if (initialized) {
    return;
  }

  initialized = true;
  unlistenZoomChanged = await listen<ZoomChangedPayload>(ZOOM_CHANGED_EVENT, (event) => {
    updateAppSettingsLocally({ zoomLevel: clampZoom(event.payload.zoomLevel) });
  });
}

export async function applySavedZoom(): Promise<void> {
  await invoke("apply_saved_zoom");
}

export async function zoomIn(): Promise<void> {
  await invoke("zoom_in");
}

export async function zoomOut(): Promise<void> {
  await invoke("zoom_out");
}

export async function resetZoom(): Promise<void> {
  await invoke("zoom_reset");
}

export function disposeZoomState(): void {
  unlistenZoomChanged?.();
  unlistenZoomChanged = null;
  initialized = false;
}
