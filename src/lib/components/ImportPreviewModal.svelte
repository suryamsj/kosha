<script lang="ts">
  import { configStore, type ImportPreview } from '$lib/config.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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

<Modal title="Import Hosts" {onClose} width="420px">
  {#if preview.accepted.length === 0 && preview.skipped.length === 0}
    <p class="text-sm text-text-muted">
      Nothing to import — no valid Host entries found in this file.
    </p>
  {:else}
    {#if preview.accepted.length > 0}
      <p class="text-sm font-medium text-text">Will import:</p>
      <ul class="mb-3 mt-1 list-disc pl-5 text-sm text-text">
        {#each preview.accepted as host (host.aliases.join(","))}
          <li>{host.aliases.join(", ")}</li>
        {/each}
      </ul>
    {/if}
    {#if preview.skipped.length > 0}
      <p class="text-sm font-medium text-text">Skipped (already exists):</p>
      <ul class="mt-1 list-disc pl-5 text-sm text-text-muted">
        {#each preview.skipped as alias (alias)}
          <li>{alias}</li>
        {/each}
      </ul>
    {/if}
  {/if}
  {#if error}
    <p class="mt-2 text-sm text-danger">{error}</p>
  {/if}
  {#snippet footer()}
    <Button onclick={onClose}>Cancel</Button>
    {#if preview.accepted.length > 0}
      <Button variant="primary" onclick={confirmImport} disabled={importing}>
        {importing ? 'Importing...' : 'Confirm Import'}
      </Button>
    {/if}
  {/snippet}
</Modal>
