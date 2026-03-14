<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { APP_TITLE, APP_VERSION } from "$lib/constants/app";
  import { appSettings, patchAppSettings, type CloseBehavior, type ThemeSetting } from "$lib/stores/app-settings";

  let isSaving = false;
  let copied = false;
  let copyError = "";
  let recentHints = "";

  const themeOptions: ThemeSetting[] = ["system", "dark", "light"];
  const closeBehaviorOptions: CloseBehavior[] = ["hide", "quit"];

  interface NavigationHintRecord {
    timestamp: number;
    source: string;
    target: string;
    classification: string;
    authFlow: boolean;
    note: string;
  }

  async function updateSetting<T extends keyof import("$lib/stores/app-settings").AppSettings>(
    key: T,
    value: import("$lib/stores/app-settings").AppSettings[T],
  ): Promise<void> {
    isSaving = true;
    try {
      await patchAppSettings({ [key]: value });
    } finally {
      isSaving = false;
    }
  }

  async function refreshHints(): Promise<void> {
    const records = await invoke<NavigationHintRecord[]>("get_recent_navigation_hints");
    recentHints = records.length
      ? records
          .map((record) => `${formatTimestamp(record.timestamp)} | ${record.source} | ${record.classification} | ${record.target} | auth=${record.authFlow} | ${record.note}`)
          .join("\n")
      : "No recent embedded navigation hints yet. Turn on Navigation debug logging, then sign in or browse inside Grok to capture a few safe redirect summaries.";
  }

  async function copyHints(): Promise<void> {
    copied = false;
    copyError = "";

    try {
      recentHints = await invoke<string>("copy_recent_navigation_hints");
      copied = true;
    } catch (error) {
      copyError = error instanceof Error ? error.message : String(error);
    }
  }

  async function clearHints(): Promise<void> {
    copyError = "";
    copied = false;
    await invoke("clear_recent_navigation_hints");
    await refreshHints();
  }

  onMount(() => {
    void refreshHints();
  });

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
</script>

<section class="settings-card" aria-label="Shell settings" data-shell-settings>
  <div class="heading-row">
    <div>
      <p class="label">Settings</p>
      <h3>Core shell behavior</h3>
    </div>
    <span class:busy={isSaving}>{isSaving ? "Saving" : "Saved"}</span>
  </div>

  <label>
    <span>Theme</span>
    <select value={$appSettings.theme} on:change={(event) => void updateSetting("theme", (event.currentTarget as HTMLSelectElement).value as ThemeSetting)}>
      {#each themeOptions as option}
        <option value={option}>{option}</option>
      {/each}
    </select>
  </label>

  <label>
    <span>Close behavior</span>
    <select value={$appSettings.closeBehavior} on:change={(event) => void updateSetting("closeBehavior", (event.currentTarget as HTMLSelectElement).value as CloseBehavior)}>
      {#each closeBehaviorOptions as option}
        <option value={option}>{option}</option>
      {/each}
    </select>
  </label>

  <label class="toggle-row">
    <span>
      <strong>Restore zoom</strong>
      <small>Reapply the saved browser-style zoom on launch.</small>
    </span>
    <input type="checkbox" checked={$appSettings.restoreZoom} on:change={(event) => void updateSetting("restoreZoom", (event.currentTarget as HTMLInputElement).checked)} />
  </label>

  <label class="toggle-row">
    <span>
      <strong>Open shell links externally</strong>
      <small>Non-Grok links clicked in the shell chrome open in the default browser.</small>
    </span>
    <input type="checkbox" checked={$appSettings.openExternalLinksInBrowser} on:change={(event) => void updateSetting("openExternalLinksInBrowser", (event.currentTarget as HTMLInputElement).checked)} />
  </label>

  <label class="toggle-row">
    <span>
      <strong>Navigation debug logging</strong>
      <small>Emit scoped navigation policy logs from the Rust policy layer.</small>
    </span>
    <input type="checkbox" checked={$appSettings.navigationDebugLogging} on:change={(event) => void updateSetting("navigationDebugLogging", (event.currentTarget as HTMLInputElement).checked)} />
  </label>

  <div class="developer-card">
    <div class="heading-row">
      <div>
        <p class="label">Developer</p>
        <h3>Recent navigation hints</h3>
      </div>
      <span>{copied ? "Copied" : copyError ? "Error" : "Ready"}</span>
    </div>

    <p class="developer-note">Sanitized recent policy decisions from the embedded Grok navigation layer.</p>

    <textarea readonly rows="6" value={recentHints}></textarea>

    <div class="developer-actions">
      <button type="button" on:click={() => void refreshHints()}>Refresh</button>
      <button type="button" on:click={() => void copyHints()}>Copy Hints</button>
      <button type="button" on:click={() => void clearHints()}>Clear</button>
    </div>

    {#if copyError}
      <p class="developer-error">{copyError}</p>
    {/if}
  </div>

  <div class="about-card" aria-label="About">
    <div class="heading-row">
      <div>
        <p class="label">About</p>
        <h3>{APP_TITLE}</h3>
      </div>
      <span>v{APP_VERSION}</span>
    </div>

    <p class="developer-note">Single-window macOS shell for Grok with local notes, prompt snippets, workspaces, diagnostics, and native browser-style zoom.</p>
  </div>
</section>

<style>
  .settings-card {
    border: 1px solid var(--panel-border);
    border-radius: 18px;
    padding: 18px;
    background: var(--panel);
    backdrop-filter: blur(16px);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    display: grid;
    gap: 14px;
  }

  .heading-row {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 12px;
  }

  .label {
    margin: 0 0 8px;
    font-size: 12px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  h3 {
    margin: 0;
    font-size: 16px;
  }

  span.busy {
    color: var(--accent);
  }

  label {
    display: grid;
    gap: 8px;
    font-size: 13px;
    color: #dce6f5;
  }

  select {
    width: 100%;
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    padding: 10px 12px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: inherit;
  }

  .toggle-row {
    grid-template-columns: 1fr auto;
    align-items: center;
    gap: 12px;
  }

  strong {
    display: block;
    font-weight: 600;
  }

  small {
    color: var(--text-muted);
    line-height: 1.4;
  }

  input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .developer-card {
    border-top: 1px solid var(--panel-border);
    padding-top: 14px;
    display: grid;
    gap: 10px;
  }

  .about-card {
    border-top: 1px solid var(--panel-border);
    padding-top: 14px;
    display: grid;
    gap: 10px;
  }

  .developer-note,
  .developer-error {
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .developer-error {
    color: #ff9b8a;
  }

  textarea {
    width: 100%;
    resize: vertical;
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    padding: 10px 12px;
    min-height: 120px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: 12px/1.4 "SF Mono", "Menlo", monospace;
  }

  .developer-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
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
</style>
