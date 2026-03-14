<script lang="ts">
  import {
    appendPromptToScratchpad,
    copyPromptContent,
    createPrompt,
    duplicatePrompt,
    deletePrompt,
    exportPromptLibrary,
    getPromptById,
    importPromptLibrary,
    promptLibrary,
    setPromptLibraryOpen,
    type PromptSnippet,
    updatePrompt,
  } from "$lib/stores/prompt-library";

  let searchTerm = "";
  let favoritesOnly = false;
  let selectedPromptId: string | null = null;
  let selectedPrompt: PromptSnippet | undefined;
  let status = "Local prompt snippets";

  $: filteredPrompts = $promptLibrary.prompts
    .filter((prompt) => {
      const query = searchTerm.trim().toLowerCase();
      if (!query) {
        return !favoritesOnly || prompt.isFavorite;
      }

      const matchesQuery =
        prompt.title.toLowerCase().includes(query) ||
        prompt.tags.some((tag) => tag.toLowerCase().includes(query));

      return matchesQuery && (!favoritesOnly || prompt.isFavorite);
    })
    .sort((left, right) => Number(right.isFavorite) - Number(left.isFavorite) || left.title.localeCompare(right.title));

  $: if (!selectedPromptId && filteredPrompts.length > 0) {
    selectedPromptId = filteredPrompts[0].id;
  }

  $: if (selectedPromptId && !filteredPrompts.some((prompt) => prompt.id === selectedPromptId)) {
    selectedPromptId = filteredPrompts[0]?.id ?? null;
  }

  $: selectedPrompt = getPromptById(selectedPromptId);

  function createNewPrompt(): void {
    selectedPromptId = createPrompt();
    status = "Prompt created";
  }

  async function importPrompts(): Promise<void> {
    try {
      status = await importPromptLibrary();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  async function exportPrompts(): Promise<void> {
    try {
      status = await exportPromptLibrary();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  async function copySelectedPrompt(): Promise<void> {
    if (!selectedPrompt) {
      return;
    }

    try {
      await copyPromptContent(selectedPrompt.content);
      status = "Prompt copied";
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  function appendSelectedPrompt(): void {
    if (!selectedPrompt) {
      return;
    }

    appendPromptToScratchpad(selectedPrompt.content);
    status = "Added to scratchpad";
  }

  function removeSelectedPrompt(): void {
    if (!selectedPrompt) {
      return;
    }

    deletePrompt(selectedPrompt.id);
    status = "Prompt deleted";
  }

  function duplicateSelectedPrompt(): void {
    if (!selectedPrompt) {
      return;
    }

    selectedPromptId = duplicatePrompt(selectedPrompt.id);
    status = "Prompt duplicated";
  }
</script>

<section class="prompt-card" aria-label="Prompt library">
  <div class="heading-row">
    <div>
      <p class="label">Prompt Library</p>
      <h3>Local snippets</h3>
    </div>
    <div class="toolbar">
      <button type="button" on:click={createNewPrompt}>New</button>
      <button type="button" on:click={() => void importPrompts()}>Import</button>
      <button type="button" on:click={() => void exportPrompts()}>Export</button>
      <button type="button" on:click={() => setPromptLibraryOpen(!$promptLibrary.isOpen)}>
        {$promptLibrary.isOpen ? "Hide" : "Show"}
      </button>
    </div>
  </div>

  {#if $promptLibrary.isOpen}
    <div class="filters">
      <input
        type="text"
        placeholder="Search prompts or tags"
        bind:value={searchTerm}
      />
      <label class="favorite-filter">
        <input type="checkbox" bind:checked={favoritesOnly} />
        <span>Favorites first</span>
      </label>
    </div>

    <div class="prompt-list" role="list">
      {#if filteredPrompts.length === 0}
        <p class="empty">
          {searchTerm.trim()
            ? "No prompts match that search yet."
            : "No local prompts yet. Create one to save reusable instructions, outlines, or starter drafts."}
        </p>
      {:else}
        {#each filteredPrompts as prompt}
          <button
            type="button"
            class:selected={prompt.id === selectedPromptId}
            class="prompt-item"
            on:click={() => {
              selectedPromptId = prompt.id;
              status = "Prompt selected";
            }}
          >
            <strong>{prompt.isFavorite ? "★ " : ""}{prompt.title}</strong>
            {#if prompt.tags.length > 0}
              <span class="tag-row">
                {#each prompt.tags as tag}
                  <small class="tag-chip">{tag}</small>
                {/each}
              </span>
            {/if}
            <span>{prompt.content.slice(0, 72) || "Empty prompt"}</span>
          </button>
        {/each}
      {/if}
    </div>

    {#if selectedPrompt}
      <div class="editor">
        <label>
          <span>Title</span>
          <input
            type="text"
            value={selectedPrompt.title}
            on:input={(event) => updatePrompt(selectedPrompt.id, { title: (event.currentTarget as HTMLInputElement).value })}
          />
        </label>

        <label>
          <span>Prompt</span>
          <textarea
            rows="8"
            value={selectedPrompt.content}
            on:input={(event) => updatePrompt(selectedPrompt.id, { content: (event.currentTarget as HTMLTextAreaElement).value })}
          ></textarea>
        </label>

        <label>
          <span>Tags</span>
          <input
            type="text"
            value={selectedPrompt.tags.join(", ")}
            placeholder="analysis, rewrite, planning"
            on:input={(event) =>
              updatePrompt(selectedPrompt.id, {
                tags: (event.currentTarget as HTMLInputElement).value
                  .split(",")
                  .map((tag) => tag.trim())
                  .filter(Boolean),
              })}
          />
        </label>

        <div class="editor-actions">
          <button type="button" on:click={() => {
            updatePrompt(selectedPrompt.id, { isFavorite: !selectedPrompt.isFavorite });
            status = selectedPrompt.isFavorite ? "Removed favorite" : "Marked favorite";
          }}>
            {selectedPrompt.isFavorite ? "Unfavorite" : "Favorite"}
          </button>
          <button type="button" on:click={duplicateSelectedPrompt}>Duplicate</button>
          <button type="button" on:click={() => void copySelectedPrompt()}>Copy</button>
          <button type="button" on:click={appendSelectedPrompt}>Add to Scratchpad</button>
          <button type="button" on:click={removeSelectedPrompt}>Delete</button>
        </div>
      </div>
    {/if}
  {:else}
    <p class="collapsed-note">Prompt Library is hidden. Use the title bar or `View -> Toggle Prompt Library` to reopen it.</p>
  {/if}

  <p class="status">{status}</p>
</section>

<style>
  .prompt-card {
    border: 1px solid var(--panel-border);
    border-radius: 18px;
    padding: 18px;
    background: var(--panel);
    backdrop-filter: blur(16px);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
    display: grid;
    gap: 12px;
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

  .toolbar,
  .editor-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .filters {
    display: grid;
    gap: 10px;
  }

  .favorite-filter {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .prompt-list {
    display: grid;
    gap: 8px;
    max-height: 220px;
    overflow: auto;
  }

  .prompt-item {
    display: grid;
    gap: 4px;
    text-align: left;
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.03);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .prompt-item.selected {
    border-color: rgba(92, 214, 255, 0.28);
    background: rgba(92, 214, 255, 0.08);
  }

  .prompt-item span,
  .empty,
  .collapsed-note,
  .status,
  label span {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .empty,
  .collapsed-note,
  .status {
    margin: 0;
  }

  .tag-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    width: fit-content;
    border: 1px solid var(--panel-border);
    border-radius: 999px;
    padding: 2px 8px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-muted);
    font-size: 11px;
    line-height: 1.4;
  }

  .editor {
    display: grid;
    gap: 10px;
  }

  label {
    display: grid;
    gap: 6px;
  }

  input,
  textarea,
  button {
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    padding: 10px 12px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: inherit;
  }

  textarea {
    resize: vertical;
    min-height: 140px;
  }

  button {
    cursor: pointer;
  }
</style>
