<script lang="ts">
  import { keysStore } from '$lib/keys.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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

<Modal title="New Key" {onClose}>
  <form onsubmit={submit} class="flex flex-col gap-3">
    <Input label="Name" placeholder="id_ed25519" bind:value={name} />
    <Input
      label="Passphrase (optional)"
      type="password"
      bind:value={passphrase}
    />
    {#if error}
      <p class="text-sm text-danger">{error}</p>
    {/if}
    <div class="mt-1 flex justify-end gap-2">
      <Button onclick={onClose}>Cancel</Button>
      <Button type="submit" variant="primary" disabled={submitting}>
        {submitting ? 'Generating...' : 'Generate'}
      </Button>
    </div>
  </form>
</Modal>
