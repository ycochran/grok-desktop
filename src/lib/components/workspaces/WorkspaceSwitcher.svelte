<script lang="ts">
  import { activeWorkspace, createWorkspace, deleteWorkspace, renameWorkspace, switchWorkspace, workspaceState } from "$lib/stores/workspaces";

  let createName = "";
  let renameName = "";
  let status = "";
  let error = "";
  let saving = false;
  let lastWorkspaceId = "";

  $: if ($activeWorkspace && $activeWorkspace.id !== lastWorkspaceId) {
    lastWorkspaceId = $activeWorkspace.id;
    renameName = $activeWorkspace.name;
  }

  $: recentWorkspaces = $workspaceState.workspaces
    .filter((workspace) => workspace.id !== $workspaceState.activeWorkspaceId)
    .sort((left, right) => right.lastUsedAt - left.lastUsedAt)
    .slice(0, 3);

  async function handleSwitch(event: Event): Promise<void> {
    const workspaceId = (event.currentTarget as HTMLSelectElement).value;
    if (!workspaceId || workspaceId === $workspaceState.activeWorkspaceId) {
      return;
    }

    saving = true;
    error = "";
    try {
      status = await switchWorkspace(workspaceId);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      saving = false;
    }
  }

  async function handleCreate(): Promise<void> {
    saving = true;
    error = "";
    try {
      status = await createWorkspace(createName);
      createName = "";
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      saving = false;
    }
  }

  async function handleRename(): Promise<void> {
    if (!$activeWorkspace) {
      return;
    }

    saving = true;
    error = "";
    try {
      status = await renameWorkspace($activeWorkspace.id, renameName);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      saving = false;
    }
  }

  async function handleDelete(): Promise<void> {
    if (!$activeWorkspace || $activeWorkspace.isDefault) {
      return;
    }

    saving = true;
    error = "";
    try {
      status = await deleteWorkspace($activeWorkspace.id);
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      saving = false;
    }
  }

  function formatLastUsed(timestamp: number): string {
    if (!timestamp) {
      return "Not used yet";
    }

    return new Date(timestamp * 1000).toLocaleString([], {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }
</script>

<section class="workspace-card" aria-label="Workspaces">
  <div class="heading-row">
    <div>
      <p class="label">Workspaces</p>
      <h3>Local shell mode</h3>
    </div>
    <span>{saving ? "Saving" : "Ready"}</span>
  </div>

  <label>
    <span>Active workspace</span>
    <select value={$workspaceState.activeWorkspaceId} on:change={(event) => void handleSwitch(event)}>
      {#each $workspaceState.workspaces as workspace}
        <option value={workspace.id}>{workspace.name}</option>
      {/each}
    </select>
  </label>

  {#if $activeWorkspace}
    <p class="workspace-meta">
      {$activeWorkspace.isDefault ? "Default workspace" : "Custom workspace"}
      <span>Last used {formatLastUsed($activeWorkspace.lastUsedAt)}</span>
    </p>
  {/if}

  {#if recentWorkspaces.length > 0}
    <div class="recent-row" aria-label="Recent workspaces">
      <span class="recent-label">Recent workspaces</span>
      {#each recentWorkspaces as workspace}
        <button
          class="recent-chip"
          title={`Last used ${formatLastUsed(workspace.lastUsedAt)}`}
          type="button"
          disabled={saving}
          on:click={() => void switchWorkspace(workspace.id).then((message) => status = message)}
        >
          <span class="chip-dot" aria-hidden="true"></span>
          {workspace.name}
        </button>
      {/each}
    </div>
  {/if}

  {#if $workspaceState.workspaces.length === 1}
    <p class="status">Start with the default workspace, then create another one when you want a separate local scratchpad layout.</p>
  {/if}

  <div class="action-grid">
    <label>
      <span>Create workspace</span>
      <input bind:value={createName} maxlength="48" placeholder="Research" type="text" />
    </label>
    <button type="button" disabled={saving} on:click={() => void handleCreate()}>Create</button>
  </div>

  {#if $activeWorkspace}
    <div class="action-grid">
      <label>
        <span>Rename active workspace</span>
        <input bind:value={renameName} maxlength="48" type="text" />
      </label>
      <button type="button" disabled={saving} on:click={() => void handleRename()}>Rename</button>
    </div>
  {/if}

  {#if $activeWorkspace && !$activeWorkspace.isDefault}
    <button class="danger" type="button" disabled={saving} on:click={() => void handleDelete()}>
      Delete Workspace
    </button>
  {/if}

  <p class:error class="status">
    {error || status || "Scratchpad content and shell panel visibility are scoped per workspace."}
  </p>
</section>

<style>
  .workspace-card {
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

  label {
    display: grid;
    gap: 8px;
    font-size: 13px;
    color: inherit;
  }

  select,
  input,
  button {
    width: 100%;
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    padding: 10px 12px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: inherit;
  }

  button {
    width: auto;
    cursor: pointer;
  }

  button:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .danger {
    color: #ffb3a6;
    border-color: rgba(255, 155, 138, 0.3);
  }

  .workspace-meta {
    margin: 0;
    display: grid;
    gap: 4px;
    color: var(--text-muted);
    font-size: 12px;
  }

  .recent-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .recent-label {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .recent-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: auto;
    padding: 5px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.03);
    font-size: 11px;
    color: var(--text-muted);
  }

  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--accent);
    opacity: 0.75;
  }

  .action-grid {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    align-items: end;
  }

  .status {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .status.error {
    color: #ffb3a6;
  }
</style>
