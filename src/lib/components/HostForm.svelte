<script lang="ts">
  import { configStore } from "$lib/config.svelte";
  import { keysStore } from "$lib/keys.svelte";
  import type { HostEntry } from "$lib/config.svelte";
  import Modal from "$lib/components/ui/Modal.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    mode,
    initial,
    onClose,
  }: { mode: "add" | "edit"; initial?: HostEntry; onClose: () => void } =
    $props();

  let aliasInput = $state(initial?.aliases.join(" ") ?? "");
  let hostName = $state(initial?.hostName ?? "");
  let user = $state(initial?.user ?? "");
  let port = $state(initial?.port ?? "");
  let identityFile = $state(initial?.identityFile ?? "");
  let error = $state<string | null>(null);
  let submitting = $state(false);

  function renderBlock(
    aliases: string[],
    hn: string,
    u: string,
    p: string,
    idf: string,
  ) {
    const lines = [`Host ${aliases.join(" ")}`];
    if (hn) lines.push(`  HostName ${hn}`);
    if (u) lines.push(`  User ${u}`);
    if (p) lines.push(`  Port ${p}`);
    if (idf) lines.push(`  IdentityFile ${idf}`);
    return lines.join("\n");
  }

  let afterPreview = $derived(
    renderBlock(
      aliasInput.trim().split(/\s+/).filter(Boolean),
      hostName,
      user,
      port,
      identityFile,
    ),
  );

  let beforePreview = $derived(
    initial
      ? renderBlock(
          initial.aliases,
          initial.hostName ?? "",
          initial.user ?? "",
          initial.port ?? "",
          initial.identityFile ?? "",
        )
      : "",
  );

  async function submit(event: Event) {
    event.preventDefault();
    const aliases = aliasInput.trim().split(/\s+/).filter(Boolean);
    if (aliases.length === 0) {
      error = "At least one alias is required";
      return;
    }
    submitting = true;
    error = null;
    try {
      if (mode === "add") {
        await configStore.addHost(
          aliases,
          hostName || null,
          user || null,
          port || null,
          identityFile || null,
        );
      } else {
        await configStore.editHost(
          initial!.aliases,
          aliases,
          hostName || null,
          user || null,
          port || null,
          identityFile || null,
        );
      }
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<Modal title={mode === "add" ? "New Host" : "Edit Host"} {onClose} width="480px">
  <form onsubmit={submit} class="flex flex-col gap-3">
    <Input label="Alias" placeholder="github gh" bind:value={aliasInput} />
    <Input label="Host Name" placeholder="github.com" bind:value={hostName} />
    <Input label="User" placeholder="git" bind:value={user} />
    <Input label="Port" placeholder="22" bind:value={port} />
    <label class="flex flex-col gap-1">
      <span class="text-sm font-medium text-text">Identity File</span>
      <select
        bind:value={identityFile}
        class="rounded-sm border border-border bg-surface px-3 py-1.5 text-sm text-text"
      >
        <option value="">None</option>
        {#each keysStore.keys as key (key.name)}
          <option value={`~/.ssh/${key.name}`}>{key.name}</option>
        {/each}
      </select>
    </label>

    <div class="flex gap-4 rounded-sm bg-canvas p-3">
      {#if mode === "edit"}
        <div class="flex-1">
          <strong class="text-xs font-medium text-text-muted">Before</strong>
          <pre class="mt-1 whitespace-pre-wrap font-mono text-xs text-text">{beforePreview}</pre>
        </div>
      {/if}
      <div class="flex-1">
        <strong class="text-xs font-medium text-text-muted"
          >{mode === "edit" ? "After" : "Preview"}</strong
        >
        <pre class="mt-1 whitespace-pre-wrap font-mono text-xs text-text">{afterPreview}</pre>
      </div>
    </div>

    {#if error}
      <p class="text-sm text-danger">{error}</p>
    {/if}

    <div class="mt-1 flex justify-end gap-2">
      <Button onclick={onClose}>Cancel</Button>
      <Button type="submit" variant="primary" disabled={submitting}>
        {submitting ? "Saving..." : "Save"}
      </Button>
    </div>
  </form>
</Modal>
