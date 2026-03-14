<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { grokSession } from "$lib/stores/grok-session";
  import { promptLibrary } from "$lib/stores/prompt-library";
  import { sidebarCollapsed } from "$lib/stores/sidebar-state";
  import { scratchpad } from "$lib/stores/scratchpad";
  import { resetZoom } from "$lib/stores/zoom-state";

  const GROK_STATE_EVENT = "app://grok-surface-state";

  interface GrokBounds {
    x: number;
    y: number;
    width: number;
    height: number;
  }

  interface GrokSurfacePayload {
    phase: "started" | "finished" | "blocked" | "auth" | "recovering";
    url: string;
    message?: string;
    isAuthFlow: boolean;
  }

  let viewportElement: HTMLDivElement | null = null;
  let isLoading = true;
  let hasError = false;
  let isAuthFlow = false;
  let statusMessage = "Loading Grok inside the embedded surface.";
  let currentUrl = "https://grok.com/";
  let animationFrameId = 0;
  let boundsSummary = "";

  function scheduleBoundsSync(): void {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = requestAnimationFrame(() => {
      animationFrameId = requestAnimationFrame(() => {
        void syncBounds();
      });
    });
  }

  async function syncBounds(): Promise<void> {
    if (!viewportElement) {
      return;
    }

    const rect = viewportElement.getBoundingClientRect();
    const width = viewportElement.clientWidth;
    const height = viewportElement.clientHeight;
    if (width < 2 || height < 2) {
      return;
    }

    const viewportWidth = document.documentElement.clientWidth || window.innerWidth;
    const viewportHeight = document.documentElement.clientHeight || window.innerHeight;
    const clampedLeft = Math.max(0, Math.min(rect.left, viewportWidth - 2));
    const clampedTop = Math.max(0, Math.min(rect.top, viewportHeight - 2));
    const clampedWidth = Math.max(2, Math.min(width, viewportWidth - clampedLeft));
    const clampedHeight = Math.max(2, Math.min(height, viewportHeight - clampedTop));

    const bounds: GrokBounds = {
      x: Math.round(clampedLeft),
      y: Math.round(clampedTop),
      width: Math.round(clampedWidth),
      height: Math.round(clampedHeight),
    };
    boundsSummary = `${bounds.width}×${bounds.height} at ${bounds.x},${bounds.y}`;

    try {
      await invoke("mount_grok_webview", { bounds });
      hasError = false;
    } catch (error) {
      console.error("Failed to mount the Grok surface.", error);
      hasError = true;
      isLoading = false;
      statusMessage = "The embedded Grok surface could not be attached.";
    }
  }

  async function reloadGrok(): Promise<void> {
    isLoading = true;
    hasError = false;
    statusMessage = "Reloading the embedded Grok surface.";

    try {
      await invoke("reload_grok_webview");
    } catch (error) {
      console.error("Failed to reload Grok.", error);
      hasError = true;
      isLoading = false;
      statusMessage = "The embedded Grok surface could not be reloaded.";
    }
  }

  async function openCurrentPageInBrowser(): Promise<void> {
    try {
      await invoke("open_grok_in_browser");
    } catch (error) {
      console.error("Failed to open the current Grok page in the browser.", error);
      hasError = true;
      statusMessage = "Opening the current Grok page in the browser failed.";
    }
  }

  function updateSurfaceState(payload: GrokSurfacePayload): void {
    currentUrl = payload.url;
    isAuthFlow = payload.isAuthFlow;

    switch (payload.phase) {
      case "started":
      case "recovering":
        isLoading = true;
        hasError = false;
        statusMessage = payload.message ?? "Loading Grok inside the embedded surface.";
        break;
      case "auth":
        isLoading = false;
        hasError = false;
        statusMessage = payload.message ?? "Authentication is required to continue in the embedded Grok surface.";
        break;
      case "blocked":
        isLoading = false;
        hasError = true;
        statusMessage = payload.message ?? "That navigation was blocked or moved to the default browser.";
        break;
      case "finished":
        isLoading = false;
        hasError = false;
        statusMessage = payload.message ?? "Embedded Grok is ready.";
        break;
    }
  }

  onMount(() => {
    const resizeObserver = new ResizeObserver(() => {
      scheduleBoundsSync();
    });
    const panelObserver = new ResizeObserver(() => {
      scheduleBoundsSync();
    });

    let unlistenState: UnlistenFn | null = null;
    const unsubscribeScratchpad = scratchpad.subscribe(() => {
      scheduleBoundsSync();
    });
    const unsubscribePromptLibrary = promptLibrary.subscribe(() => {
      scheduleBoundsSync();
    });
    const unsubscribeSidebar = sidebarCollapsed.subscribe(() => {
      scheduleBoundsSync();
    });

    if (viewportElement) {
      resizeObserver.observe(viewportElement);
      if (viewportElement.parentElement instanceof HTMLElement) {
        panelObserver.observe(viewportElement.parentElement);
      }
      const shellContent = viewportElement.closest(".shell-content");
      if (shellContent instanceof HTMLElement) {
        panelObserver.observe(shellContent);
      }
    }

    void (async () => {
      unlistenState = await listen<GrokSurfacePayload>(GROK_STATE_EVENT, (event) => {
        updateSurfaceState(event.payload);
      });

      scheduleBoundsSync();
    })();

    window.addEventListener("resize", scheduleBoundsSync);

    return () => {
      window.removeEventListener("resize", scheduleBoundsSync);
      resizeObserver.disconnect();
       panelObserver.disconnect();
      unlistenState?.();
      unsubscribeScratchpad();
      unsubscribePromptLibrary();
      unsubscribeSidebar();
      cancelAnimationFrame(animationFrameId);
    };
  });
</script>

<div class="surface">
  <header class="toolbar">
    <div>
      <p class="label">Grok surface</p>
      <h2>Embedded session shell</h2>
    </div>

    <div class="actions">
      <button type="button" on:click={reloadGrok}>Reload</button>
      <button type="button" on:click={openCurrentPageInBrowser}>Open in Browser</button>
      <button type="button" on:click={() => void resetZoom()}>Reset Zoom</button>
    </div>
  </header>

  <div class="surface-meta">
    <span class="url">{currentUrl}</span>
    <span>{isAuthFlow ? "Auth flow" : hasError ? "Attention needed" : isLoading ? "Loading" : "Ready"} · {boundsSummary || "Sizing..."}</span>
  </div>

  <div class="surface-note">{statusMessage}</div>

  {#if $grokSession.status === "signing-in" || $grokSession.status === "attention" || $grokSession.status === "error"}
    <div class="session-banner" data-tone={$grokSession.status}>
      <span>{$grokSession.message}</span>
      <button type="button" on:click={reloadGrok}>{$grokSession.status === "signing-in" ? "Retry Auth" : "Reload Grok"}</button>
    </div>
  {/if}

  <div class="webview-host" bind:this={viewportElement}>
    {#if isLoading}
      <div class="overlay loading">
        <h3>Loading Grok</h3>
        <p>{statusMessage}</p>
      </div>
    {/if}

    {#if !isLoading && isAuthFlow}
      <div class="corner-note auth-note">
        <strong>Auth flow active</strong>
        <span>{statusMessage}</span>
      </div>
    {/if}

    {#if hasError}
      <div class="overlay error">
        <h3>Navigation moved out of the embedded surface</h3>
        <p>{statusMessage}</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .surface {
    flex: 1;
    min-height: 0;
    border: 1px solid var(--panel-border);
    border-radius: 24px;
    padding: 18px;
    background:
      linear-gradient(160deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.01)),
      var(--chrome-strong);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.04),
      0 24px 60px rgba(0, 0, 0, 0.22);
    display: grid;
    grid-template-rows: auto auto auto minmax(0, 1fr);
    gap: 14px;
    min-width: 0;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 16px;
    min-width: 0;
  }

  .label {
    margin: 0 0 6px;
    font-size: 12px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--accent);
  }

  h2 {
    margin: 0;
    font-size: 24px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 10px;
    flex-shrink: 0;
  }

  button {
    border: 1px solid var(--panel-border);
    border-radius: 999px;
    padding: 10px 14px;
    background: rgba(255, 255, 255, 0.04);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .surface-meta,
  .surface-note,
  .session-banner {
    color: var(--text-muted);
    font-size: 12px;
  }

  .surface-meta {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    min-width: 0;
  }

  .url {
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .session-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border: 1px solid var(--panel-border);
    border-radius: 14px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.03);
  }

  .session-banner[data-tone="signing-in"] {
    border-color: rgba(92, 214, 255, 0.24);
  }

  .session-banner[data-tone="attention"],
  .session-banner[data-tone="error"] {
    border-color: rgba(255, 211, 123, 0.24);
  }

  .webview-host {
    position: relative;
    min-height: 0;
    min-width: 0;
    min-height: 420px;
    border-radius: 18px;
    overflow: hidden;
    border: 1px solid rgba(92, 214, 255, 0.1);
    background:
      linear-gradient(135deg, rgba(7, 16, 27, 0.92), rgba(7, 16, 27, 0.7)),
      rgba(0, 0, 0, 0.2);
  }

  .overlay {
    position: absolute;
    inset: 0;
    z-index: 2;
    display: grid;
    place-content: center;
    gap: 8px;
    padding: 24px;
    text-align: center;
    background: rgba(5, 10, 18, 0.84);
    backdrop-filter: blur(8px);
  }

  .overlay h3 {
    margin: 0;
    font-size: 20px;
  }

  .overlay p {
    margin: 0;
    max-width: 540px;
    color: #c5d0e1;
    line-height: 1.5;
  }

  .error {
    background: rgba(43, 10, 10, 0.88);
  }

  .corner-note {
    position: absolute;
    top: 14px;
    right: 14px;
    z-index: 2;
    max-width: min(320px, calc(100% - 28px));
    display: grid;
    gap: 4px;
    padding: 10px 12px;
    border: 1px solid rgba(92, 214, 255, 0.22);
    border-radius: 14px;
    background: rgba(13, 25, 44, 0.78);
    color: #d7e4f8;
    font-size: 12px;
    line-height: 1.4;
    pointer-events: none;
    backdrop-filter: blur(8px);
  }

  .corner-note strong {
    font-size: 12px;
  }

  @media (max-width: 840px) {
    .toolbar,
    .surface-meta {
      align-items: start;
      flex-direction: column;
    }

    .actions {
      justify-content: start;
    }
  }
</style>
