<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { KeyInfo } from "$lib/keys.svelte";

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

<div class="modal-backdrop">
  <div class="modal">
    <h2>{keyInfo.name}</h2>
    <p>Type: {keyInfo.keyType}</p>
    <p>Fingerprint: {keyInfo.fingerprint}</p>
    {#if error}
      <p class="error">{error}</p>
    {:else}
      <textarea readonly rows="4">{publicKey}</textarea>
      <button onclick={copy}>{copied ? "Copied!" : "Copy public key"}</button>
    {/if}
    <div class="actions">
      <button onclick={onClose}>Close</button>
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
    min-width: 400px;
  }
  textarea {
    width: 100%;
    font-family: monospace;
    font-size: 0.8rem;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 0.75rem;
  }
  .error {
    color: #c0392b;
  }
</style>
