<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    configStore,
    type HostEntry,
    type ImportPreview,
  } from "$lib/config.svelte";
  import { tagsStore } from "$lib/tags.svelte";
  import HostForm from "$lib/components/HostForm.svelte";
  import DeleteHostConfirm from "$lib/components/DeleteHostConfirm.svelte";
  import ImportPreviewModal from "$lib/components/ImportPreviewModal.svelte";

  let showAddHost = $state(false);
  let editingHost = $state<HostEntry | null>(null);
  let hostToDelete = $state<HostEntry | null>(null);
  let hostSearch = $state("");
  let tagFilter = $state("");

  type TestResult = {
    status: "testing" | "success" | "error";
    message: string;
  };
  let testResults = $state<Record<string, TestResult>>({});
  let tagInputs = $state<Record<string, string>>({});

  let fileInput = $state<HTMLInputElement | null>(null);
  let importError = $state<string | null>(null);
  let importText = $state("");
  let importPreview = $state<ImportPreview | null>(null);

  onMount(() => {
    configStore.refresh();
    tagsStore.refresh();
  });

  let allTags = $derived.by(() => {
    const set = new Set<string>();
    for (const tags of Object.values(tagsStore.tags)) {
      for (const tag of tags) set.add(tag);
    }
    return [...set].sort();
  });

  $effect(() => {
    if (tagFilter && !allTags.includes(tagFilter)) {
      tagFilter = "";
    }
  });

  let filteredHosts = $derived.by(() => {
    const q = hostSearch.trim().toLowerCase();
    return configStore.hosts.filter((host) => {
      const matchesSearch =
        !q ||
        host.aliases.some((alias) => alias.toLowerCase().includes(q)) ||
        (host.hostName?.toLowerCase().includes(q) ?? false);
      const matchesTag =
        !tagFilter ||
        (tagsStore.tags[host.aliases.join(",")] ?? []).includes(tagFilter);
      return matchesSearch && matchesTag;
    });
  });

  async function testConnection(host: HostEntry) {
    const key = host.aliases.join(",");
    testResults[key] = { status: "testing", message: "" };
    try {
      const message = await invoke<string>("test_connection", {
        alias: host.aliases[0],
      });
      testResults[key] = { status: "success", message };
    } catch (e) {
      testResults[key] = { status: "error", message: String(e) };
    }
  }

  function tagsDisplayValue(host: HostEntry): string {
    const key = host.aliases.join(",");
    if (key in tagInputs) return tagInputs[key];
    return (tagsStore.tags[key] ?? []).join(", ");
  }

  function onTagsInput(host: HostEntry, value: string) {
    tagInputs[host.aliases.join(",")] = value;
  }

  async function onTagsBlur(host: HostEntry) {
    const key = host.aliases.join(",");
    const value = tagInputs[key] ?? "";
    const parsed = value
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    await tagsStore.setTags(key, parsed);
    delete tagInputs[key];
  }

  async function exportHosts() {
    const text = await configStore.exportHosts();
    const blob = new Blob([text], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `kosha-hosts-export-${new Date().toISOString().replace(/[:.]/g, "-")}.conf`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function triggerImport() {
    fileInput?.click();
  }

  async function onFileSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;

    importError = null;
    try {
      const text = await file.text();
      const preview = await configStore.previewImport(text);
      importText = text;
      importPreview = preview;
    } catch (e) {
      importError = String(e);
    }
  }
</script>

<div class="header">
  <input
    class="search"
    type="search"
    placeholder="Search hosts..."
    bind:value={hostSearch}
  />
  <select bind:value={tagFilter}>
    <option value="">All tags</option>
    {#each allTags as tag (tag)}
      <option value={tag}>{tag}</option>
    {/each}
  </select>
  <div class="actions">
    <button onclick={exportHosts}>Export</button>
    <button onclick={triggerImport}>Import</button>
    <button onclick={() => (showAddHost = true)}>Add Host</button>
  </div>
</div>

<input
  bind:this={fileInput}
  type="file"
  accept=".conf,.txt,text/plain"
  class="hidden-file-input"
  onchange={onFileSelected}
/>

{#if tagsStore.error}
  <p class="error">{tagsStore.error}</p>
{/if}

{#if importError}
  <p class="error">{importError}</p>
{/if}

{#if configStore.error}
  <p class="error">{configStore.error}</p>
{:else if configStore.hosts.length === 0}
  <p class="empty-state">No hosts configured.</p>
{:else if filteredHosts.length === 0}
  <p class="empty-state">No hosts match the current filters.</p>
{:else}
  <table>
    <thead>
      <tr>
        <th>Alias</th>
        <th>Host Name</th>
        <th>User</th>
        <th>Port</th>
        <th>Identity File</th>
        <th>Tags</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each filteredHosts as host (host.aliases.join(","))}
        {@const result = testResults[host.aliases.join(",")]}
        <tr>
          <td>{host.aliases.join(", ")}</td>
          <td>{host.hostName ?? "-"}</td>
          <td>{host.user ?? "-"}</td>
          <td>{host.port ?? "-"}</td>
          <td class="mono">{host.identityFile ?? "-"}</td>
          <td>
            <input
              class="tags-input"
              type="text"
              placeholder="work, personal"
              value={tagsDisplayValue(host)}
              oninput={(e) =>
                onTagsInput(host, (e.target as HTMLInputElement).value)}
              onblur={() => onTagsBlur(host)}
            />
          </td>
          <td>
            <button onclick={() => (editingHost = host)}>Edit</button>
            <button onclick={() => (hostToDelete = host)}>Delete</button>
            <button
              onclick={() => testConnection(host)}
              disabled={result?.status === "testing"}
            >
              {result?.status === "testing" ? "Testing..." : "Test"}
            </button>
            {#if result && result.status !== "testing"}
              <span
                class={result.status === "success" ? "test-ok" : "test-fail"}
              >
                {result.status === "success" ? "✓" : "✗"}
                {result.message}
              </span>
            {/if}
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

{#if importPreview}
  <ImportPreviewModal
    text={importText}
    preview={importPreview}
    onClose={() => (importPreview = null)}
  />
{/if}

<style>
  .header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }
  .search {
    flex: 1;
    max-width: 240px;
    padding: 0.5rem;
    box-sizing: border-box;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }
  .hidden-file-input {
    display: none;
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
  .tags-input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.3rem;
    font-size: 0.85rem;
  }
  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: #666;
  }
  .error {
    color: #c0392b;
  }
  .test-ok {
    color: #2e7d32;
    margin-left: 0.5rem;
    font-size: 0.8rem;
  }
  .test-fail {
    color: #c0392b;
    margin-left: 0.5rem;
    font-size: 0.8rem;
  }
</style>
