<script lang="ts">
  import CommandPalette from "$lib/components/shell/CommandPalette.svelte";
  import ScratchpadPanel from "$lib/components/scratchpad/ScratchpadPanel.svelte";
  import GrokSurface from "$lib/components/grok/GrokSurface.svelte";
  import Sidebar from "$lib/components/shell/Sidebar.svelte";
  import StatusBar from "$lib/components/shell/StatusBar.svelte";
  import TitleBar from "$lib/components/shell/TitleBar.svelte";
  import { sidebarCollapsed } from "$lib/stores/sidebar-state";
  import { scratchpad } from "$lib/stores/scratchpad";
</script>

<div class="app-frame">
  <CommandPalette />
  <TitleBar />
  <div class:sidebar-collapsed={$sidebarCollapsed} class="shell-body">
    {#if !$sidebarCollapsed}
      <Sidebar />
    {/if}
    <main class="shell-content" aria-label="Grok workspace">
      <GrokSurface />
    </main>
    {#if $scratchpad.isOpen}
      <ScratchpadPanel />
    {/if}
  </div>
  <StatusBar />
</div>

<style>
  :global(:root) {
    color-scheme: dark;
    font-family: "SF Pro Text", "Segoe UI", sans-serif;
    background:
      radial-gradient(circle at top, rgba(74, 111, 255, 0.16), transparent 36%),
      linear-gradient(180deg, #0d1320 0%, #091019 100%);
    color: #e7edf7;
    --chrome: rgba(10, 17, 28, 0.86);
    --chrome-strong: rgba(14, 24, 40, 0.96);
    --panel: rgba(19, 31, 51, 0.78);
    --panel-border: rgba(145, 171, 214, 0.18);
    --text-muted: #8ea1c1;
    --accent: #5cd6ff;
    --accent-soft: rgba(92, 214, 255, 0.18);
  }

  :global(:root[data-theme="light"]) {
    color-scheme: light;
    background:
      radial-gradient(circle at top, rgba(59, 130, 246, 0.12), transparent 36%),
      linear-gradient(180deg, #eef4fb 0%, #dfe8f4 100%);
    color: #0f2036;
    --chrome: rgba(245, 249, 255, 0.82);
    --chrome-strong: rgba(255, 255, 255, 0.96);
    --panel: rgba(255, 255, 255, 0.78);
    --panel-border: rgba(49, 88, 140, 0.16);
    --text-muted: #5c7392;
    --accent: #006fe8;
    --accent-soft: rgba(0, 111, 232, 0.1);
  }

  :global(html),
  :global(body),
  :global(#app) {
    margin: 0;
    min-height: 100%;
    height: 100%;
  }

  :global(body) {
    overflow: hidden;
  }

  .app-frame {
    height: 100vh;
    display: grid;
    grid-template-rows: auto 1fr auto;
    background: transparent;
  }

  .shell-body {
    min-height: 0;
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr) auto;
    gap: 16px;
    padding: 16px;
    overflow: hidden;
  }

  .shell-body.sidebar-collapsed {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .shell-content {
    min-width: 0;
    min-height: 0;
    display: flex;
    overflow: hidden;
  }

  @media (max-width: 960px) {
    .shell-body {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }

    .shell-body.sidebar-collapsed {
      grid-template-columns: 1fr;
    }
  }
</style>
