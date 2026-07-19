<script lang="ts">
  import { configStore } from '$lib/config.svelte';
  import type { HostEntry } from '$lib/config.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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

<Modal title={`Delete "${host.aliases.join(', ')}"?`} {onClose}>
  <p class="mb-3 text-sm text-text-muted">
    This permanently removes this Host block from ~/.ssh/config. A backup is
    made first.
  </p>
  <pre class="whitespace-pre-wrap rounded-sm bg-canvas p-2 font-mono text-xs text-text">{renderBlock(host)}</pre>
  {#if error}
    <p class="mt-2 text-sm text-danger">{error}</p>
  {/if}
  {#snippet footer()}
    <Button onclick={onClose}>Cancel</Button>
    <Button variant="danger" onclick={confirmDelete} disabled={deleting}>
      {deleting ? 'Deleting...' : 'Delete'}
    </Button>
  {/snippet}
</Modal>
