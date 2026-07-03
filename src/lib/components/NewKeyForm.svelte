<script lang="ts">
  import { keysStore } from "$lib/keys.svelte";

  let { onClose }: { onClose: () => void } = $props();

  let name = $state("");
  let passphrase = $state("");
  let error = $state<string | null>(null);
  let submitting = $state(false);

  async function submit(event: Event) {
    event.preventDefault();
    if (!name.trim()) {
      error = "Name is required";
      return;
    }
    submitting = true;
    error = null;
    try {
      await keysStore.generate(name.trim(), passphrase);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="modal-backdrop">
  <div class="modal">
    <h2>New Key</h2>
    <form onsubmit={submit}>
      <label>
        Name
        <input bind:value={name} placeholder="id_ed25519" />
      </label>
      <label>
        Passphrase (optional)
        <input type="password" bind:value={passphrase} />
      </label>
      {#if error}
        <p class="error">{error}</p>
      {/if}
      <div class="actions">
        <button type="button" onclick={onClose}>Cancel</button>
        <button type="submit" disabled={submitting}>
          {submitting ? "Generating..." : "Generate"}
        </button>
      </div>
    </form>
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
  label {
    display: block;
    margin-bottom: 0.75rem;
  }
  input {
    width: 100%;
    margin-top: 0.25rem;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .error {
    color: #c0392b;
  }
</style>
