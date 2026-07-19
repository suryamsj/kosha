<script lang="ts">
  import { keysStore } from '$lib/keys.svelte';
  import type { KeyInfo } from '$lib/keys.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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

<Modal title={`Delete "${keyInfo.name}"?`} {onClose}>
  <p class="text-sm text-text-muted">
    This permanently deletes the key pair. This cannot be undone.
  </p>
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
