<script lang="ts">
  import { scratchpad, clearScratchpad, copyScratchpadContent, exportScratchpad, importScratchpad, setScratchpadContent, setScratchpadOpen } from "$lib/stores/scratchpad";

  let status = "Autosaves locally";

  async function copyContent(): Promise<void> {
    try {
      await copyScratchpadContent($scratchpad.content);
      status = "Copied";
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  async function importContent(): Promise<void> {
    try {
      status = await importScratchpad();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  async function exportContent(): Promise<void> {
    try {
      status = await exportScratchpad($scratchpad.content);
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }
</script>

<aside class="scratchpad-panel" aria-label="Scratchpad">
  <header class="scratchpad-header">
    <div>
      <p class="label">Scratchpad</p>
      <h3>Local notes</h3>
    </div>
    <div class="actions">
      <button type="button" on:click={() => void importContent()}>Import</button>
      <button type="button" on:click={() => void exportContent()}>Export</button>
      <button type="button" on:click={() => void copyContent()}>Copy</button>
      <button
        type="button"
        on:click={() => {
          clearScratchpad();
          status = "Cleared";
        }}
      >
        Clear
      </button>
      <button type="button" on:click={() => void setScratchpadOpen(false)}>Hide</button>
    </div>
  </header>

  <p class="note">Saved locally in the desktop shell. Not shared with the embedded Grok webview.</p>

  {#if !$scratchpad.content.trim()}
    <p class="empty-state">Start with a quick note, capture a prompt draft, or paste links and follow-up tasks. Everything here stays local to this workspace.</p>
  {/if}

  <textarea
    class="editor"
    value={$scratchpad.content}
    placeholder="Capture quick notes, prompts, links, or follow-up tasks."
    on:input={(event) => setScratchpadContent((event.currentTarget as HTMLTextAreaElement).value)}
  ></textarea>

  <footer class="scratchpad-footer">
    <span>{status}</span>
    <span>{$scratchpad.content.length} chars</span>
  </footer>
</aside>

<style>
  .scratchpad-panel {
    min-width: 320px;
    max-width: 360px;
    min-height: 0;
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    gap: 12px;
    border: 1px solid var(--panel-border);
    border-radius: 24px;
    padding: 18px;
    background:
      linear-gradient(160deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.01)),
      var(--chrome-strong);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.04),
      0 24px 60px rgba(0, 0, 0, 0.18);
    overflow: hidden;
  }

  .scratchpad-header {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 12px;
  }

  .label {
    margin: 0 0 6px;
    font-size: 12px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--accent);
  }

  h3 {
    margin: 0;
    font-size: 20px;
  }

  .note,
  .empty-state,
  .scratchpad-footer {
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
  }

  button {
    border: 1px solid var(--panel-border);
    border-radius: 999px;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.04);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .editor {
    width: 100%;
    min-height: 0;
    resize: none;
    border: 1px solid var(--panel-border);
    border-radius: 18px;
    padding: 14px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: 13px/1.5 "SF Mono", "Menlo", monospace;
  }

  .scratchpad-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  @media (max-width: 1200px) {
    .scratchpad-panel {
      max-width: none;
      min-width: 0;
    }
  }
</style>
