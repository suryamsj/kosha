<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { PencilSimple, Trash, DownloadSimple, UploadSimple, Plus } from "phosphor-svelte";
  import {
    configStore,
    type HostEntry,
    type ImportPreview,
  } from "$lib/config.svelte";
  import { tagsStore } from "$lib/tags.svelte";
  import HostForm from "$lib/components/HostForm.svelte";
  import DeleteHostConfirm from "$lib/components/DeleteHostConfirm.svelte";
  import ImportPreviewModal from "$lib/components/ImportPreviewModal.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";

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

<div class="mb-4 flex items-center gap-2">
  <div class="max-w-xs flex-1">
    <Input type="search" placeholder="Search hosts..." bind:value={hostSearch} />
  </div>
  <select
    bind:value={tagFilter}
    class="rounded-sm border border-border bg-surface px-3 py-1.5 text-sm text-text"
  >
    <option value="">All tags</option>
    {#each allTags as tag (tag)}
      <option value={tag}>{tag}</option>
    {/each}
  </select>
  <div class="ml-auto flex gap-2">
    <Button onclick={exportHosts}>
      <span class="inline-flex items-center gap-1">
        <DownloadSimple size={16} weight="bold" /> Export
      </span>
    </Button>
    <Button onclick={triggerImport}>
      <span class="inline-flex items-center gap-1">
        <UploadSimple size={16} weight="bold" /> Import
      </span>
    </Button>
    <Button variant="primary" onclick={() => (showAddHost = true)}>
      <span class="inline-flex items-center gap-1">
        <Plus size={16} weight="bold" /> Add Host
      </span>
    </Button>
  </div>
</div>

<input
  bind:this={fileInput}
  type="file"
  accept=".conf,.txt,text/plain"
  class="hidden"
  onchange={onFileSelected}
/>

{#if tagsStore.error}
  <p class="mb-2 text-sm text-danger">{tagsStore.error}</p>
{/if}

{#if importError}
  <p class="mb-2 text-sm text-danger">{importError}</p>
{/if}

{#if configStore.error}
  <p class="text-sm text-danger">{configStore.error}</p>
{:else if configStore.hosts.length === 0}
  <p class="py-12 text-center text-text-muted">No hosts configured.</p>
{:else if filteredHosts.length === 0}
  <p class="py-12 text-center text-text-muted">
    No hosts match the current filters.
  </p>
{:else}
  <table class="w-full border-collapse text-sm">
    <thead>
      <tr>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">Alias</th>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">Host Name</th>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">User</th>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">Port</th>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">Identity File</th>
        <th class="border-b border-border px-2 py-2 text-left text-text-muted">Tags</th>
        <th class="border-b border-border px-2 py-2"></th>
      </tr>
    </thead>
    <tbody>
      {#each filteredHosts as host (host.aliases.join(","))}
        {@const result = testResults[host.aliases.join(",")]}
        <tr class="hover:bg-canvas">
          <td class="border-b border-border px-2 py-2 font-mono text-text">{host.aliases.join(", ")}</td>
          <td class="border-b border-border px-2 py-2 text-text">{host.hostName ?? "-"}</td>
          <td class="border-b border-border px-2 py-2 text-text">{host.user ?? "-"}</td>
          <td class="border-b border-border px-2 py-2 text-text">{host.port ?? "-"}</td>
          <td class="border-b border-border px-2 py-2 font-mono text-xs text-text-muted">{host.identityFile ?? "-"}</td>
          <td class="border-b border-border px-2 py-2">
            <input
              type="text"
              placeholder="work, personal"
              value={tagsDisplayValue(host)}
              oninput={(e) =>
                onTagsInput(host, (e.target as HTMLInputElement).value)}
              onblur={() => onTagsBlur(host)}
              class="w-full rounded-sm border border-border bg-surface px-2 py-1 text-xs text-text"
            />
          </td>
          <td class="border-b border-border px-2 py-2">
            <div class="flex items-center gap-1">
              <IconButton icon={PencilSimple} label="Edit host" onclick={() => (editingHost = host)} />
              <IconButton icon={Trash} label="Delete host" variant="danger" onclick={() => (hostToDelete = host)} />
              <Button
                onclick={() => testConnection(host)}
                disabled={result?.status === "testing"}
              >
                {result?.status === "testing" ? "Testing..." : "Test"}
              </Button>
              {#if result && result.status !== "testing"}
                <Badge variant={result.status === "success" ? "success" : "danger"}>
                  {result.message}
                </Badge>
              {/if}
            </div>
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
