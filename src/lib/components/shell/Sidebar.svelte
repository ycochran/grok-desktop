<script lang="ts">
  import PromptLibrarySection from "$lib/components/prompts/PromptLibrarySection.svelte";
  import ShellSettings from "$lib/components/settings/ShellSettings.svelte";
  import WorkspaceSwitcher from "$lib/components/workspaces/WorkspaceSwitcher.svelte";

  const sections = ["Workspaces", "Settings", "Prompt Library", "Diagnostics"];
</script>

<aside class="sidebar" aria-label="Navigation">
  <div class="sidebar-card">
    <p class="label">Shell</p>
    <ul>
      {#each sections as section}
        <li>{section}</li>
      {/each}
    </ul>
  </div>

  <WorkspaceSwitcher />

  <ShellSettings />

  <PromptLibrarySection />

  <div class="sidebar-card muted">
    <p class="label">Notes</p>
    <p>The embedded Grok surface keeps auth and Grok-related navigation inside the child webview while sending unrelated pages to the default browser.</p>
  </div>
</aside>

<style>
  .sidebar {
    display: grid;
    align-content: start;
    gap: 16px;
    min-height: 0;
    overflow: auto;
    padding-right: 4px;
  }

  .sidebar-card {
    border: 1px solid var(--panel-border);
    border-radius: 18px;
    padding: 18px;
    background: var(--panel);
    backdrop-filter: blur(16px);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
  }

  .label {
    margin: 0 0 12px;
    font-size: 12px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 10px;
  }

  li {
    padding: 10px 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.03);
  }

  .muted p:last-child {
    margin: 0;
    color: var(--text-muted);
    line-height: 1.5;
  }
</style>
