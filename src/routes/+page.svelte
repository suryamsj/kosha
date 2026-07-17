<script lang="ts">
  import { onMount } from "svelte";
  import { keysStore, type KeyInfo } from "$lib/keys.svelte";
  import NewKeyForm from "$lib/components/NewKeyForm.svelte";
  import KeyDetail from "$lib/components/KeyDetail.svelte";
  import DeleteConfirm from "$lib/components/DeleteConfirm.svelte";
  import HostsTable from "$lib/components/HostsTable.svelte";

  let showNewKeyForm = $state(false);
  let selectedKey = $state<KeyInfo | null>(null);
  let keyToDelete = $state<KeyInfo | null>(null);
  let activeTab = $state<"keys" | "hosts">("keys");
  let keySearch = $state("");

  onMount(() => {
    keysStore.refresh();
  });

  function formatDate(secs: number) {
    if (!secs) return "unknown";
    return new Date(secs * 1000).toLocaleDateString();
  }

  let filteredKeys = $derived.by(() => {
    const q = keySearch.trim().toLowerCase();
    if (!q) return keysStore.keys;
    return keysStore.keys.filter(
      (key) =>
        key.name.toLowerCase().includes(q) ||
        key.associatedHosts.some((host) => host.toLowerCase().includes(q)),
    );
  });
</script>

<main class="container">
  <div class="header">
    <h1>Kosha</h1>
    {#if activeTab === "keys"}
      <button onclick={() => (showNewKeyForm = true)}>New Key</button>
    {/if}
  </div>

  <div class="tabs">
    <button
      class:active={activeTab === "keys"}
      onclick={() => (activeTab = "keys")}
    >
      Keys
    </button>
    <button
      class:active={activeTab === "hosts"}
      onclick={() => (activeTab = "hosts")}
    >
      Hosts
    </button>
  </div>

  {#if activeTab === "keys"}
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
      <input
        class="search"
        type="search"
        placeholder="Search keys..."
        bind:value={keySearch}
      />
      {#if filteredKeys.length === 0}
        <p class="empty-state">No keys match "{keySearch}".</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Fingerprint</th>
              <th>Created</th>
              <th>Used by</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each filteredKeys as key (key.name)}
              <tr onclick={() => (selectedKey = key)}>
                <td>{key.name}</td>
                <td>{key.keyType}</td>
                <td class="mono">{key.fingerprint}</td>
                <td>{formatDate(key.createdAt)}</td>
                <td>{key.associatedHosts.join(", ") || "-"}</td>
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
    {/if}
  {:else}
    <HostsTable />
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
    margin-bottom: 1rem;
  }
  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    border-bottom: 1px solid #ddd;
  }
  .tabs button {
    background: none;
    border: none;
    padding: 0.5rem 1rem;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }
  .tabs button.active {
    border-bottom-color: #396cd8;
    font-weight: 600;
  }
  .search {
    width: 100%;
    padding: 0.5rem;
    margin-bottom: 1rem;
    box-sizing: border-box;
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
