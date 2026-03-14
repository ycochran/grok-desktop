<script lang="ts">
  import { onMount } from "svelte";
  import { APP_TITLE } from "$lib/constants/app";
  import AppFrame from "$lib/components/shell/AppFrame.svelte";
  import { disposeGrokSessionState, initializeGrokSessionState } from "$lib/stores/grok-session";
  import { appSettings, initializeAppSettings } from "$lib/stores/app-settings";
  import { initializePromptLibrary } from "$lib/stores/prompt-library";
  import { initializeScratchpad } from "$lib/stores/scratchpad";
  import { disposeWorkspaceState, initializeWorkspaceState } from "$lib/stores/workspaces";
  import { applySavedZoom, initializeZoomState } from "$lib/stores/zoom-state";
  import { initializeWindowShell } from "$lib/tauri/window";

  onMount(() => {
    document.title = APP_TITLE;

    const unsubscribe = appSettings.subscribe((settings) => {
      document.documentElement.dataset.theme = settings.theme;
      document.title = APP_TITLE;
    });

    void (async () => {
      await initializeAppSettings();
      await initializeZoomState();
      await initializeGrokSessionState();
      await initializeWorkspaceState();
      await initializePromptLibrary();
      await initializeScratchpad();
      await applySavedZoom();
      initializeWindowShell();
    })();

    return () => {
      disposeGrokSessionState();
      disposeWorkspaceState();
      unsubscribe();
    };
  });
</script>

<AppFrame />
