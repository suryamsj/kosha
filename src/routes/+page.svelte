<script lang="ts">
  import { onMount } from "svelte";
  import { keysStore, type KeyInfo } from "$lib/keys.svelte";
  import NewKeyForm from "$lib/components/NewKeyForm.svelte";
  import KeyDetail from "$lib/components/KeyDetail.svelte";
  import DeleteConfirm from "$lib/components/DeleteConfirm.svelte";

  let showNewKeyForm = $state(false);
  let selectedKey = $state<KeyInfo | null>(null);
  let keyToDelete = $state<KeyInfo | null>(null);

  onMount(() => {
    keysStore.refresh();
  });

  function formatDate(secs: number) {
    if (!secs) return "unknown";
    return new Date(secs * 1000).toLocaleDateString();
  }
</script>

<main class="container">
  <div class="header">
    <h1>Kosha</h1>
    <button onclick={() => (showNewKeyForm = true)}>New Key</button>
  </div>

  {#if keysStore.error}
    <p class="error">{keysStore.error}</p>
  {:else if keysStore.sshDirMissing}
    <div class="empty-state">
      <p>No ~/.ssh directory found.</p>
      <button onclick={() => keysStore.createSshDir()}>Create ~/.ssh</button>
    </div>
  {:else if keysStore.keys.length === 0}
    <p class="empty-state">No keys yet.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Fingerprint</th>
          <th>Created</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each keysStore.keys as key (key.name)}
          <tr onclick={() => (selectedKey = key)}>
            <td>{key.name}</td>
            <td>{key.keyType}</td>
            <td class="mono">{key.fingerprint}</td>
            <td>{formatDate(key.createdAt)}</td>
            <td>
              <button
                onclick={(e) => {
                  e.stopPropagation();
                  keyToDelete = key;
                }}
              >
                Delete
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</main>

{#if showNewKeyForm}
  <NewKeyForm onClose={() => (showNewKeyForm = false)} />
{/if}

{#if selectedKey}
  <KeyDetail keyInfo={selectedKey} onClose={() => (selectedKey = null)} />
{/if}

{#if keyToDelete}
  <DeleteConfirm keyInfo={keyToDelete} onClose={() => (keyToDelete = null)} />
{/if}

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    text-align: left;
    padding: 0.5rem;
    border-bottom: 1px solid #ddd;
  }
  .mono {
    font-family: monospace;
    font-size: 0.8rem;
  }
  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: #666;
  }
  .error {
    color: #c0392b;
  }
</style>
