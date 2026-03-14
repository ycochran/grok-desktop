import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import { appSettings } from "$lib/stores/app-settings";

let initialized = false;

export function initializeWindowShell(): void {
  if (initialized) {
    return;
  }

  initialized = true;
  document.addEventListener("click", handleExternalLinkClick);
}

async function handleExternalLinkClick(event: MouseEvent): Promise<void> {
  const target = event.target;
  if (!(target instanceof Element)) {
    return;
  }

  const anchor = target.closest("a[href]");
  if (!(anchor instanceof HTMLAnchorElement)) {
    return;
  }

  const href = anchor.href;
  const shouldOpenExternally = get(appSettings).openExternalLinksInBrowser;
  if (!shouldOpenExternally || !isExternalHttpUrl(href)) {
    return;
  }

  event.preventDefault();
  await invoke("open_external_link", { url: href });
}

function isExternalHttpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}
