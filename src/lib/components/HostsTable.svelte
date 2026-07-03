<script lang="ts">
  import { onMount } from "svelte";
  import { configStore } from "$lib/config.svelte";

  onMount(() => {
    configStore.refresh();
  });
</script>

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
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
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
