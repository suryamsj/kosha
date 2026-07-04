<script lang="ts">
  import { configStore } from "$lib/config.svelte";
  import type { HostEntry } from "$lib/config.svelte";

  let { host, onClose }: { host: HostEntry; onClose: () => void } = $props();

  let error = $state<string | null>(null);
  let deleting = $state(false);

  function renderBlock(h: HostEntry) {
    const lines = [`Host ${h.aliases.join(" ")}`];
    if (h.hostName) lines.push(`  HostName ${h.hostName}`);
    if (h.user) lines.push(`  User ${h.user}`);
    if (h.port) lines.push(`  Port ${h.port}`);
    if (h.identityFile) lines.push(`  IdentityFile ${h.identityFile}`);
    return lines.join("\n");
  }

  async function confirmDelete() {
    deleting = true;
    error = null;
    try {
      await configStore.removeHost(host.aliases);
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
    <h2>Delete "{host.aliases.join(", ")}"?</h2>
    <p>
      This permanently removes this Host block from ~/.ssh/config. A
      backup is made first.
    </p>
    <pre>{renderBlock(host)}</pre>
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
  pre {
    background: #f6f6f6;
    padding: 0.5rem;
    font-size: 0.75rem;
    white-space: pre-wrap;
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
