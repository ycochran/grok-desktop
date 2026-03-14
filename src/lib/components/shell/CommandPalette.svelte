<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { get } from "svelte/store";
  import { onMount, tick } from "svelte";
  import { closeCommandPalette, commandPalette, openCommandPalette, recentCommandIds, recordRecentCommand, toggleCommandPalette } from "$lib/stores/command-palette";
  import { grokSession } from "$lib/stores/grok-session";
  import { promptLibrary, appendPromptToScratchpad, exportPromptLibrary, importPromptLibrary, setPromptLibraryOpen, togglePromptLibrary } from "$lib/stores/prompt-library";
  import { sidebarCollapsed, toggleSidebarCollapsed } from "$lib/stores/sidebar-state";
  import { clearScratchpadRemotely, exportScratchpad, importScratchpad, scratchpad, setScratchpadOpen, toggleScratchpad } from "$lib/stores/scratchpad";
  import { activeWorkspace, createWorkspace, deleteWorkspace, renameWorkspace, workspaceState, switchWorkspace } from "$lib/stores/workspaces";

  const COMMAND_PALETTE_EVENT = "app://command-palette";
  const GROUP_ORDER = ["Commands", "Workspaces", "Favorite Prompts"] as const;

  type CommandGroup = (typeof GROUP_ORDER)[number];

  interface CommandEntry {
    id: string;
    group: CommandGroup;
    title: string;
    keywords: string;
    shortcut?: string;
    recentIndex?: number;
    workspaceRank?: number;
    aliases?: string[];
    run: () => Promise<void> | void;
  }

  interface CommandSection {
    title: string;
    items: CommandEntry[];
  }

  let query = "";
  let selectedIndex = 0;
  let inputElement: HTMLInputElement | null = null;
  let status = "";

  $: commands = buildCommands();
  $: rankedCommands = rankCommands(commands, query, $recentCommandIds);
  $: recentCommands = !query.trim() ? rankedCommands.filter((command) => (command.recentIndex ?? -1) >= 0).slice(0, 4) : [];
  $: groupedSections = groupCommands(rankedCommands);
  $: sections = recentCommands.length > 0 ? [{ title: "Recent", items: recentCommands }, ...groupedSections] : groupedSections;
  $: flattenedCommands = sections.flatMap((section) => section.items);
  $: if (selectedIndex >= flattenedCommands.length) {
    selectedIndex = Math.max(0, flattenedCommands.length - 1);
  }

  $: if ($commandPalette) {
    void tick().then(() => {
      inputElement?.focus();
      inputElement?.select();
    });
  } else {
    query = "";
    selectedIndex = 0;
  }

  function buildCommands(): CommandEntry[] {
    const workspaceName = normalizedWorkspaceName(query);
    const currentWorkspace = get(activeWorkspace);
    const workspaces = get(workspaceState).workspaces;

    const workspaceCommands = workspaces.map<CommandEntry>((workspace, index) => ({
      id: `workspace.switch:${workspace.id}`,
      group: "Workspaces",
      title:
        workspace.id === get(workspaceState).activeWorkspaceId
          ? `Current Workspace: ${workspace.name}`
          : `Switch Workspace: ${workspace.name}`,
      keywords: `${workspace.name} workspace switch local shell ${workspace.isDefault ? "default" : "custom"}`,
      aliases: ["ws", "workspace", "switch"],
      workspaceRank: index,
      run: async () => {
        if (workspace.id !== get(workspaceState).activeWorkspaceId) {
          status = await switchWorkspace(workspace.id);
        }
      },
    }));

    const promptCommands = get(promptLibrary).prompts
      .filter((prompt) => prompt.isFavorite)
      .map<CommandEntry>((prompt) => ({
        id: `favorite-prompt:${prompt.id}`,
        group: "Favorite Prompts",
        title: `Add Favorite Prompt: ${prompt.title}`,
        keywords: `${prompt.title} ${prompt.tags.join(" ")} favorite prompt scratchpad`,
        aliases: ["fp", "favorite"],
        run: () => {
          appendPromptToScratchpad(prompt.content);
          setScratchpadOpen(true);
        },
      }));

    return [
      {
        id: "scratchpad.toggle",
        group: "Commands",
        title: get(scratchpad).isOpen ? "Hide Scratchpad" : "Show Scratchpad",
        keywords: "scratchpad notes toggle panel",
        aliases: ["sp", "notes"],
        shortcut: "Cmd+Shift+K",
        run: async () => {
          await toggleScratchpad();
        },
      },
      {
        id: "prompt-library.toggle",
        group: "Commands",
        title: get(promptLibrary).isOpen ? "Hide Prompt Library" : "Show Prompt Library",
        keywords: "prompt library snippets toggle panel",
        aliases: ["prompt", "prompts", "pl"],
        shortcut: "Cmd+Shift+P",
        run: async () => {
          await togglePromptLibrary();
        },
      },
      {
        id: "sidebar.toggle",
        group: "Commands",
        title: get(sidebarCollapsed) ? "Show Sidebar" : "Hide Sidebar",
        keywords: "sidebar shell navigation focus mode toggle",
        aliases: ["sidebar", "nav", "focus"],
        run: () => {
          toggleSidebarCollapsed();
        },
      },
      {
        id: "settings.open",
        group: "Commands",
        title: "Open Settings",
        keywords: "settings preferences shell",
        aliases: ["prefs", "preferences"],
        run: () => {
          document.querySelector("[data-shell-settings]")?.scrollIntoView({ behavior: "smooth", block: "start" });
        },
      },
      {
        id: "workspace.create",
        group: "Workspaces",
        title: workspaceName ? `Create Workspace: ${workspaceName}` : "Create Workspace",
        keywords: `workspace create new local shell ${workspaceName}`,
        aliases: ["ws", "workspace", "new"],
        run: async () => {
          status = await createWorkspace(workspaceName || `Workspace ${workspaces.length + 1}`);
        },
      },
      {
        id: "workspace.rename",
        group: "Workspaces",
        title:
          currentWorkspace && workspaceName
            ? `Rename Workspace to: ${workspaceName}`
            : `Rename Workspace: ${currentWorkspace?.name ?? "Current Workspace"}`,
        keywords: `workspace rename current local shell ${currentWorkspace?.name ?? ""} ${workspaceName}`,
        aliases: ["ws", "workspace", "rename"],
        run: async () => {
          if (!currentWorkspace) {
            return;
          }

          status = await renameWorkspace(currentWorkspace.id, workspaceName || currentWorkspace.name);
        },
      },
      ...(currentWorkspace && !currentWorkspace.isDefault
        ? [
            {
              id: `workspace.delete:${currentWorkspace.id}`,
              group: "Workspaces" as const,
              title: `Delete Workspace: ${currentWorkspace.name}`,
              keywords: `${currentWorkspace.name} workspace delete remove local shell`,
              aliases: ["ws", "workspace", "delete", "remove"],
              run: async () => {
                status = await deleteWorkspace(currentWorkspace.id);
              },
            },
          ]
        : []),
      {
        id: "grok.reload",
        group: "Commands",
        title: "Reload Grok",
        keywords: "grok reload refresh recover",
        aliases: ["refresh"],
        run: async () => {
          await invoke("reload_grok_webview");
        },
      },
      {
        id: "grok.retry",
        group: "Commands",
        title: get(grokSession).status === "signing-in" ? "Retry Grok Auth Flow" : "Recover Grok Surface",
        keywords: "grok retry auth recover surface",
        aliases: ["retry", "recover"],
        run: async () => {
          await invoke("reload_grok_webview");
        },
      },
      {
        id: "diagnostics.copy",
        group: "Commands",
        title: "Copy Navigation Diagnostics",
        keywords: "navigation diagnostics debug copy hints",
        aliases: ["diag", "nav", "debug"],
        run: async () => {
          await invoke("copy_recent_navigation_hints");
        },
      },
      {
        id: "diagnostics.clear",
        group: "Commands",
        title: "Clear Navigation Diagnostics",
        keywords: "navigation diagnostics debug clear hints",
        aliases: ["diag", "nav", "debug"],
        run: async () => {
          await invoke("clear_recent_navigation_hints");
        },
      },
      {
        id: "scratchpad.clear",
        group: "Commands",
        title: "Clear Scratchpad",
        keywords: "scratchpad clear notes",
        aliases: ["sp", "notes"],
        run: async () => {
          await clearScratchpadRemotely();
        },
      },
      {
        id: "scratchpad.import",
        group: "Commands",
        title: "Import Scratchpad",
        keywords: "scratchpad import text notes",
        aliases: ["sp", "notes"],
        run: async () => {
          status = await importScratchpad();
        },
      },
      {
        id: "scratchpad.export",
        group: "Commands",
        title: "Export Scratchpad",
        keywords: "scratchpad export text notes",
        aliases: ["sp", "notes"],
        run: async () => {
          status = await exportScratchpad(get(scratchpad).content);
        },
      },
      {
        id: "prompt-library.import",
        group: "Commands",
        title: "Import Prompt Library",
        keywords: "prompt library import snippets",
        aliases: ["prompt", "pl"],
        run: async () => {
          status = await importPromptLibrary();
          setPromptLibraryOpen(true);
        },
      },
      {
        id: "prompt-library.export",
        group: "Commands",
        title: "Export Prompt Library",
        keywords: "prompt library export snippets",
        aliases: ["prompt", "pl"],
        run: async () => {
          status = await exportPromptLibrary();
        },
      },
      ...workspaceCommands,
      ...promptCommands,
    ];
  }

  function rankCommands(commandsToRank: CommandEntry[], currentQuery: string, recentIds: string[]): CommandEntry[] {
    return commandsToRank
      .map((command) => ({
        ...command,
        recentIndex: recentIds.indexOf(command.id),
        score: scoreCommand(command, currentQuery, recentIds.indexOf(command.id)),
      }))
      .filter((command) => command.score > Number.NEGATIVE_INFINITY)
      .sort((left, right) => {
        if (right.score !== left.score) {
          return right.score - left.score;
        }

        return left.title.localeCompare(right.title);
      })
      .map(({ score: _score, ...command }) => command);
  }

  function scoreCommand(command: CommandEntry, currentQuery: string, recentIndex: number): number {
    const normalizedQuery = currentQuery.trim().toLowerCase();
    const title = command.title.toLowerCase();
    const keywords = command.keywords.toLowerCase();
    const aliases = (command.aliases ?? []).join(" ").toLowerCase();
    const combined = `${title} ${keywords} ${aliases}`;
    let score = 0;

    if (!normalizedQuery) {
      score += command.group === "Commands" ? 24 : 12;
    } else {
      const terms = normalizedQuery.split(/\s+/).filter(Boolean);
      if (!terms.every((term) => combined.includes(term))) {
        return Number.NEGATIVE_INFINITY;
      }

      score += 10;

      if (title === normalizedQuery) {
        score += 180;
      } else if (title.startsWith(normalizedQuery)) {
        score += 130;
      } else if (title.includes(normalizedQuery)) {
        score += 80;
      } else if (keywords.includes(normalizedQuery)) {
        score += 40;
      }

      for (const term of terms) {
        if (title.split(/\s+/).some((word) => word.startsWith(term))) {
          score += 24;
        } else if (title.includes(term)) {
          score += 14;
        } else if (keywords.includes(term)) {
          score += 8;
        }
      }
    }

    if (recentIndex >= 0) {
      score += 72 - recentIndex * 8;
    }

    if (command.group === "Workspaces") {
      score += normalizedQuery ? 18 : 10;
      if ((command.workspaceRank ?? -1) >= 0) {
        score += Math.max(0, 22 - command.workspaceRank! * 5);
      }
    }

    if (command.group === "Favorite Prompts") {
      score += normalizedQuery ? 10 : 6;
    }

    return score;
  }

  function groupCommands(commandsToGroup: CommandEntry[]): CommandSection[] {
    return GROUP_ORDER.map((title) => ({
      title,
      items: commandsToGroup.filter((command) => command.group === title),
    })).filter((section) => section.items.length > 0);
  }

  async function executeSelected(): Promise<void> {
    const command = flattenedCommands[selectedIndex];
    if (!command) {
      return;
    }

    try {
      await command.run();
      recordRecentCommand(command.id);
      closeCommandPalette();
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    }
  }

  function handleGlobalKeydown(event: KeyboardEvent): void {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      toggleCommandPalette();
      return;
    }

    if (!$commandPalette) {
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      closeCommandPalette();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, Math.max(0, flattenedCommands.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selectedIndex = Math.max(0, selectedIndex - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      void executeSelected();
    }
  }

  onMount(() => {
    let unlistenMenu: UnlistenFn | null = null;

    window.addEventListener("keydown", handleGlobalKeydown);

    void (async () => {
      unlistenMenu = await listen<string>(COMMAND_PALETTE_EVENT, (event) => {
        openCommandPalette();
        query = typeof event.payload === "string" ? event.payload : "";
      });
    })();

    return () => {
      window.removeEventListener("keydown", handleGlobalKeydown);
      unlistenMenu?.();
    };
  });

  function normalizedWorkspaceName(value: string): string {
    const trimmed = value
      .replace(/^(ws|workspaces?)\b/i, "")
      .replace(/^switch workspace\b/i, "")
      .replace(/^create workspace\b/i, "")
      .replace(/^rename workspace\b/i, "")
      .trim();

    return trimmed.slice(0, 48);
  }

  function getHighlightedParts(text: string, currentQuery: string): Array<{ value: string; match: boolean }> {
    const terms = currentQuery
      .trim()
      .toLowerCase()
      .split(/\s+/)
      .filter(Boolean)
      .sort((left, right) => right.length - left.length);

    if (terms.length === 0) {
      return [{ value: text, match: false }];
    }

    const lower = text.toLowerCase();
    const matches: Array<{ start: number; end: number }> = [];

    for (const term of terms) {
      let searchIndex = 0;
      while (searchIndex < lower.length) {
        const foundIndex = lower.indexOf(term, searchIndex);
        if (foundIndex === -1) {
          break;
        }

        matches.push({ start: foundIndex, end: foundIndex + term.length });
        searchIndex = foundIndex + term.length;
      }
    }

    if (matches.length === 0) {
      return [{ value: text, match: false }];
    }

    matches.sort((left, right) => left.start - right.start);
    const merged: Array<{ start: number; end: number }> = [];
    for (const match of matches) {
      const previous = merged[merged.length - 1];
      if (!previous || match.start > previous.end) {
        merged.push(match);
      } else {
        previous.end = Math.max(previous.end, match.end);
      }
    }

    const parts: Array<{ value: string; match: boolean }> = [];
    let cursor = 0;
    for (const match of merged) {
      if (match.start > cursor) {
        parts.push({ value: text.slice(cursor, match.start), match: false });
      }
      parts.push({ value: text.slice(match.start, match.end), match: true });
      cursor = match.end;
    }
    if (cursor < text.length) {
      parts.push({ value: text.slice(cursor), match: false });
    }
    return parts;
  }
</script>

{#if $commandPalette}
  <div
    class="palette-backdrop"
    role="presentation"
    tabindex="-1"
    on:click={(event) => {
      if (event.target === event.currentTarget) {
        closeCommandPalette();
      }
    }}
    on:keydown={(event) => {
      if (event.key === "Escape") {
        closeCommandPalette();
      }
    }}
  >
    <div class="palette" role="dialog" aria-label="Command palette" aria-modal="true" tabindex="-1">
      <input
        bind:this={inputElement}
        bind:value={query}
        class="palette-input"
        placeholder="Search shell commands"
        type="text"
      />

      <div class="hint-row" aria-label="Command palette hints">
        <span><strong>Cmd+K</strong> opens. Try <code>ws</code>, <code>sp</code>, <code>pl</code>, or <code>diag</code>.</span>
        <span>Enter runs, Esc closes, ↑↓ moves</span>
      </div>

      <div class="palette-list" role="listbox" aria-label="Command results">
        {#if flattenedCommands.length === 0}
          <p class="empty">No matching commands.</p>
        {:else}
          {#each sections as section}
            <section class="palette-section" aria-label={section.title}>
              <header class="section-header">{section.title}</header>
              {#each section.items as command}
                <button
                  type="button"
                  class:selected={flattenedCommands[selectedIndex]?.id === command.id}
                  class="palette-item"
                  on:click={() => {
                    selectedIndex = flattenedCommands.findIndex((entry) => entry.id === command.id);
                    void executeSelected();
                  }}
                >
                  <span class="item-text">
                    <strong>
                      {#each getHighlightedParts(command.title, query) as part}
                        <span class:match={part.match}>{part.value}</span>
                      {/each}
                    </strong>
                    <span>{command.keywords}</span>
                  </span>
                  <span class="item-meta">
                    {#if section.title !== "Recent" && (command.recentIndex ?? -1) >= 0}
                      <small>Recent</small>
                    {/if}
                    {#if command.shortcut}
                      <kbd>{command.shortcut}</kbd>
                    {/if}
                  </span>
                </button>
              {/each}
            </section>
          {/each}
        {/if}
      </div>

      <footer class="palette-footer">
        <span>{status || "Enter runs the selected shell action"}</span>
        <span>Esc closes, ↑↓ moves</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: grid;
    place-items: start center;
    padding-top: 10vh;
    background: rgba(5, 10, 18, 0.42);
    backdrop-filter: blur(8px);
  }

  .palette {
    width: min(680px, calc(100vw - 32px));
    border: 1px solid var(--panel-border);
    border-radius: 24px;
    padding: 16px;
    background:
      linear-gradient(160deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.01)),
      var(--chrome-strong);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.04),
      0 24px 60px rgba(0, 0, 0, 0.28);
    display: grid;
    gap: 12px;
  }

  .palette-input,
  .palette-item {
    width: 100%;
    border: 1px solid var(--panel-border);
    border-radius: 14px;
    padding: 12px 14px;
    background: rgba(5, 11, 19, 0.8);
    color: inherit;
    font: inherit;
  }

  .palette-list {
    display: grid;
    gap: 8px;
    max-height: 360px;
    overflow: auto;
  }

  .hint-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .hint-row strong,
  .hint-row code {
    color: inherit;
    font: inherit;
  }

  .palette-section {
    display: grid;
    gap: 8px;
  }

  .section-header {
    padding: 0 4px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .palette-section:first-child .section-header {
    color: var(--accent);
  }

  .palette-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    text-align: left;
    cursor: pointer;
  }

  .item-text {
    min-width: 0;
    display: grid;
    gap: 4px;
  }

  .item-text span {
    overflow: hidden;
    color: var(--text-muted);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-text strong .match {
    color: var(--accent);
  }

  .item-meta {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .item-meta small {
    padding: 4px 8px;
    border: 1px solid var(--panel-border);
    border-radius: 999px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  kbd {
    border: 1px solid var(--panel-border);
    border-radius: 8px;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.04);
    font: inherit;
    font-size: 12px;
  }

  .palette-item.selected {
    border-color: rgba(92, 214, 255, 0.4);
    background:
      linear-gradient(180deg, rgba(92, 214, 255, 0.08), rgba(92, 214, 255, 0.03)),
      rgba(5, 11, 19, 0.88);
  }

  .empty,
  .palette-footer {
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.4;
  }

  .empty {
    margin: 0;
    padding: 12px 4px;
  }

  .palette-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  @media (max-width: 720px) {
    .hint-row,
    .palette-footer {
      flex-direction: column;
    }
  }
</style>
