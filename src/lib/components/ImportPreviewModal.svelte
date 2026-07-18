<script lang="ts">
  import { configStore, type ImportPreview } from "$lib/config.svelte";

  let {
    text,
    preview,
    onClose,
  }: { text: string; preview: ImportPreview; onClose: () => void } = $props();

  let error = $state<string | null>(null);
  let importing = $state(false);

  async function confirmImport() {
    importing = true;
    error = null;
    try {
      await configStore.importHosts(text);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <h2>Import Hosts</h2>
    {#if preview.accepted.length === 0 && preview.skipped.length === 0}
      <p>Nothing to import — no valid Host entries found in this file.</p>
    {:else}
      {#if preview.accepted.length > 0}
        <p><strong>Will import:</strong></p>
        <ul>
          {#each preview.accepted as host (host.aliases.join(","))}
            <li>{host.aliases.join(", ")}</li>
          {/each}
        </ul>
      {/if}
      {#if preview.skipped.length > 0}
        <p><strong>Skipped (already exists):</strong></p>
        <ul>
          {#each preview.skipped as alias (alias)}
            <li>{alias}</li>
          {/each}
        </ul>
      {/if}
    {/if}
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <div class="actions">
      <button type="button" onclick={onClose}>Cancel</button>
      {#if preview.accepted.length > 0}
        <button onclick={confirmImport} disabled={importing}>
          {importing ? "Importing..." : "Confirm Import"}
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .modal {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    min-width: 360px;
    max-height: 80vh;
    overflow-y: auto;
  }
  ul {
    margin: 0.25rem 0 0.75rem;
    padding-left: 1.25rem;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .error {
    color: #c0392b;
  }
</style>
