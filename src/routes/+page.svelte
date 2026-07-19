<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Sun, Moon } from 'phosphor-svelte';
  import { keysStore, type KeyInfo } from '$lib/keys.svelte';
  import { themeStore } from '$lib/theme.svelte';
  import NewKeyForm from '$lib/components/NewKeyForm.svelte';
  import KeyDetail from '$lib/components/KeyDetail.svelte';
  import DeleteConfirm from '$lib/components/DeleteConfirm.svelte';
  import HostsTable from '$lib/components/HostsTable.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import Input from '$lib/components/ui/Input.svelte';

  let showNewKeyForm = $state(false);
  let selectedKey = $state<KeyInfo | null>(null);
  let keyToDelete = $state<KeyInfo | null>(null);
  let activeTab = $state<'keys' | 'hosts'>('keys');
  let keySearch = $state('');

  onMount(() => {
    keysStore.refresh();
  });

  function formatDate(secs: number) {
    if (!secs) return 'unknown';
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

<main class="mx-auto max-w-3xl p-8">
  <div class="mb-4 flex items-center justify-between">
    <h1 class="text-xl font-semibold text-text">Kosha</h1>
    <div class="flex items-center gap-2">
      <IconButton
        icon={themeStore.theme === 'dark' ? Sun : Moon}
        label="Toggle theme"
        onclick={themeStore.toggle}
      />
      {#if activeTab === 'keys'}
        <Button variant="primary" onclick={() => (showNewKeyForm = true)}>
          <span class="inline-flex items-center gap-1">
            <Plus size={16} weight="bold" /> New Key
          </span>
        </Button>
      {/if}
    </div>
  </div>

  <div class="mb-6 flex gap-1 border-b border-border">
    <button
      class="border-b-2 px-4 py-2 text-sm font-medium {activeTab === 'keys'
        ? 'border-text text-text'
        : 'border-transparent text-text-muted hover:text-text'}"
      onclick={() => (activeTab = 'keys')}
    >
      Keys
    </button>
    <button
      class="border-b-2 px-4 py-2 text-sm font-medium {activeTab === 'hosts'
        ? 'border-text text-text'
        : 'border-transparent text-text-muted hover:text-text'}"
      onclick={() => (activeTab = 'hosts')}
    >
      Hosts
    </button>
  </div>

  {#if activeTab === 'keys'}
    {#if keysStore.error}
      <p class="text-sm text-danger">{keysStore.error}</p>
    {:else if keysStore.sshDirMissing}
      <div class="py-12 text-center text-text-muted">
        <p class="mb-3">No ~/.ssh directory found.</p>
        <Button onclick={() => keysStore.createSshDir()}>Create ~/.ssh</Button>
      </div>
    {:else if keysStore.keys.length === 0}
      <p class="py-12 text-center text-text-muted">No keys yet.</p>
    {:else}
      <div class="mb-4 max-w-xs">
        <Input type="search" placeholder="Search keys..." bind:value={keySearch} />
      </div>
      {#if filteredKeys.length === 0}
        <p class="py-12 text-center text-text-muted">
          No keys match "{keySearch}".
        </p>
      {:else}
        <table class="w-full border-collapse text-sm">
          <thead>
            <tr>
              <th class="border-b border-border px-2 py-2 text-left text-text-muted"
                >Name</th
              >
              <th class="border-b border-border px-2 py-2 text-left text-text-muted"
                >Type</th
              >
              <th class="border-b border-border px-2 py-2 text-left text-text-muted"
                >Fingerprint</th
              >
              <th class="border-b border-border px-2 py-2 text-left text-text-muted"
                >Created</th
              >
              <th class="border-b border-border px-2 py-2 text-left text-text-muted"
                >Used by</th
              >
              <th class="border-b border-border px-2 py-2"></th>
            </tr>
          </thead>
          <tbody>
            {#each filteredKeys as key (key.name)}
              <tr
                class="cursor-pointer hover:bg-canvas"
                onclick={() => (selectedKey = key)}
              >
                <td class="border-b border-border px-2 py-2 text-text">{key.name}</td>
                <td class="border-b border-border px-2 py-2 text-text">{key.keyType}</td>
                <td class="border-b border-border px-2 py-2 font-mono text-xs text-text-muted"
                  >{key.fingerprint}</td
                >
                <td class="border-b border-border px-2 py-2 text-text"
                  >{formatDate(key.createdAt)}</td
                >
                <td class="border-b border-border px-2 py-2 text-text"
                  >{key.associatedHosts.join(', ') || '-'}</td
                >
                <td class="border-b border-border px-2 py-2">
                  <Button
                    variant="danger"
                    onclick={(e) => {
                      e.stopPropagation();
                      keyToDelete = key;
                    }}
                  >
                    Delete
                  </Button>
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
