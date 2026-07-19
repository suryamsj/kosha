<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { KeyInfo } from '$lib/keys.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  let { keyInfo, onClose }: { keyInfo: KeyInfo; onClose: () => void } =
    $props();

  let publicKey = $state("");
  let error = $state<string | null>(null);
  let copied = $state(false);

  async function load() {
    try {
      publicKey = await invoke<string>("get_public_key", {
        name: keyInfo.name,
      });
    } catch (e) {
      error = String(e);
    }
  }
  load();

  async function copy() {
    await navigator.clipboard.writeText(publicKey);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<Modal title={keyInfo.name} {onClose} width="480px">
  <p class="text-sm text-text-muted">Type: {keyInfo.keyType}</p>
  <p class="mb-3 font-mono text-xs text-text-muted">
    Fingerprint: {keyInfo.fingerprint}
  </p>
  {#if error}
    <p class="text-sm text-danger">{error}</p>
  {:else}
    <textarea
      readonly
      rows="4"
      value={publicKey}
      class="w-full rounded-sm border border-border bg-canvas p-2 font-mono text-xs text-text"
    ></textarea>
    <div class="mt-2">
      <Button onclick={copy}>{copied ? 'Copied!' : 'Copy public key'}</Button>
    </div>
  {/if}
  {#snippet footer()}
    <Button onclick={onClose}>Close</Button>
  {/snippet}
</Modal>
