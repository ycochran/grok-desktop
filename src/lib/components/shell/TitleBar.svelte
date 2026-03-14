<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { APP_TITLE } from "$lib/constants/app";
  import { grokSession, grokSessionTone } from "$lib/stores/grok-session";
  import { promptLibrary, setPromptLibraryOpen } from "$lib/stores/prompt-library";
  import { sidebarCollapsed, toggleSidebarCollapsed } from "$lib/stores/sidebar-state";
  import { scratchpad, setScratchpadOpen } from "$lib/stores/scratchpad";
  import { activeWorkspace } from "$lib/stores/workspaces";

  $: sessionActionLabel =
    $grokSessionTone === "signing-in"
      ? "Retry Auth"
      : $grokSessionTone === "attention" || $grokSessionTone === "error"
        ? "Reload Grok"
        : "";

  async function retrySession(): Promise<void> {
    await invoke("reload_grok_webview");
  }
</script>

<div class="titlebar">
  <div class="title-copy" data-tauri-drag-region>
    <p class="eyebrow">macOS shell</p>
    <h1>{APP_TITLE}</h1>
    <p class="subtitle" title={$grokSession.message}>{$grokSession.message}</p>
  </div>
  <div class="meta">
    <span class="pill session" data-tone={$grokSessionTone} title={$grokSession.message}>{$grokSession.label}</span>
    {#if sessionActionLabel}
      <button class="pill action" type="button" on:click={() => void retrySession()}>{sessionActionLabel}</button>
    {/if}
    <button class="pill action" type="button" on:click={toggleSidebarCollapsed}>
      {$sidebarCollapsed ? "Show Sidebar" : "Hide Sidebar"}
    </button>
    <button class="pill action" type="button" on:click={() => setScratchpadOpen(!$scratchpad.isOpen)}>
      {$scratchpad.isOpen ? "Hide Scratchpad" : "Show Scratchpad"}
    </button>
    <button class="pill action" type="button" on:click={() => setPromptLibraryOpen(!$promptLibrary.isOpen)}>
      {$promptLibrary.isOpen ? "Hide Prompt Library" : "Show Prompt Library"}
    </button>
    {#if $activeWorkspace}
      <span class="pill">Workspace: {$activeWorkspace.name}</span>
    {/if}
    <span class="pill">macOS</span>
    <span class="pill">Single window</span>
  </div>
</div>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 18px 20px 14px;
    background: linear-gradient(180deg, rgba(9, 18, 31, 0.94), rgba(9, 18, 31, 0.72));
    border-bottom: 1px solid var(--panel-border);
    backdrop-filter: blur(18px);
  }

  .eyebrow {
    margin: 0 0 4px;
    font-size: 11px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .subtitle {
    margin: 6px 0 0;
    max-width: 520px;
    color: var(--text-muted);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
  }

  .pill {
    border: 1px solid var(--panel-border);
    border-radius: 999px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.03);
  }

  .action {
    cursor: pointer;
    font: inherit;
  }

  .session[data-tone="signed-in"] {
    color: #8ff0c0;
    border-color: rgba(143, 240, 192, 0.28);
    background: rgba(143, 240, 192, 0.08);
  }

  .session[data-tone="signing-in"],
  .session[data-tone="loading"] {
    color: var(--accent);
    border-color: rgba(92, 214, 255, 0.24);
    background: rgba(92, 214, 255, 0.08);
  }

  .session[data-tone="attention"] {
    color: #ffd37b;
    border-color: rgba(255, 211, 123, 0.3);
    background: rgba(255, 211, 123, 0.08);
  }

  .session[data-tone="error"] {
    color: #ff9b8a;
    border-color: rgba(255, 155, 138, 0.3);
    background: rgba(255, 155, 138, 0.08);
  }
</style>
