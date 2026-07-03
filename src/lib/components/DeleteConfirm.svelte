<script lang="ts">
  import { keysStore } from "$lib/keys.svelte";
  import type { KeyInfo } from "$lib/keys.svelte";

  let { keyInfo, onClose }: { keyInfo: KeyInfo; onClose: () => void } =
    $props();

  let error = $state<string | null>(null);
  let deleting = $state(false);

  async function confirmDelete() {
    deleting = true;
    error = null;
    try {
      await keysStore.remove(keyInfo.name);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      deleting = false;
    }
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <h2>Delete "{keyInfo.name}"?</h2>
    <p>This permanently deletes the key pair. This cannot be undone.</p>
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <div class="actions">
      <button onclick={onClose}>Cancel</button>
      <button onclick={confirmDelete} disabled={deleting} class="danger">
        {deleting ? "Deleting..." : "Delete"}
      </button>
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
    min-width: 320px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .danger {
    background: #c0392b;
    color: white;
  }
  .error {
    color: #c0392b;
  }
</style>
