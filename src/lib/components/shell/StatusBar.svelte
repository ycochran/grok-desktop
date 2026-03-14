<script lang="ts">
  import { appSettings } from "$lib/stores/app-settings";
  import { zoomLevelLabel } from "$lib/stores/zoom-state";
</script>

<footer class="statusbar">
  <span>Theme: {$appSettings.theme}</span>
  <span>On close: {$appSettings.closeBehavior}</span>
  <span>Zoom: {$zoomLevelLabel}</span>
</footer>

<style>
  .statusbar {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 16px 12px;
    border-top: 1px solid var(--panel-border);
    background: rgba(7, 14, 24, 0.9);
    color: var(--text-muted);
    font-size: 12px;
  }

  @media (max-width: 720px) {
    .statusbar {
      flex-direction: column;
    }
  }
</style>
