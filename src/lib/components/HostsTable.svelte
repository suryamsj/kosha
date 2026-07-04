<script lang="ts">
  import { onMount } from "svelte";
  import { configStore, type HostEntry } from "$lib/config.svelte";
  import HostForm from "$lib/components/HostForm.svelte";
  import DeleteHostConfirm from "$lib/components/DeleteHostConfirm.svelte";

  let showAddHost = $state(false);
  let editingHost = $state<HostEntry | null>(null);
  let hostToDelete = $state<HostEntry | null>(null);

  onMount(() => {
    configStore.refresh();
  });
</script>

<div class="header">
  <button onclick={() => (showAddHost = true)}>Add Host</button>
</div>

{#if configStore.error}
  <p class="error">{configStore.error}</p>
{:else if configStore.hosts.length === 0}
  <p class="empty-state">No hosts configured.</p>
{:else}
  <table>
    <thead>
      <tr>
        <th>Alias</th>
        <th>Host Name</th>
        <th>User</th>
        <th>Port</th>
        <th>Identity File</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each configStore.hosts as host (host.aliases.join(","))}
        <tr>
          <td>{host.aliases.join(", ")}</td>
          <td>{host.hostName ?? "-"}</td>
          <td>{host.user ?? "-"}</td>
          <td>{host.port ?? "-"}</td>
          <td class="mono">{host.identityFile ?? "-"}</td>
          <td>
            <button onclick={() => (editingHost = host)}>Edit</button>
            <button onclick={() => (hostToDelete = host)}>Delete</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

{#if showAddHost}
  <HostForm mode="add" onClose={() => (showAddHost = false)} />
{/if}

{#if editingHost}
  <HostForm
    mode="edit"
    initial={editingHost}
    onClose={() => (editingHost = null)}
  />
{/if}

{#if hostToDelete}
  <DeleteHostConfirm
    host={hostToDelete}
    onClose={() => (hostToDelete = null)}
  />
{/if}

<style>
  .header {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 1rem;
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
