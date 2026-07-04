<script lang="ts">
  import { configStore } from "$lib/config.svelte";
  import { keysStore } from "$lib/keys.svelte";
  import type { HostEntry } from "$lib/config.svelte";

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

<div class="modal-backdrop">
  <div class="modal">
    <h2>{mode === "add" ? "New Host" : "Edit Host"}</h2>
    <form onsubmit={submit}>
      <label>
        Alias
        <input bind:value={aliasInput} placeholder="github gh" />
      </label>
      <label>
        Host Name
        <input bind:value={hostName} placeholder="github.com" />
      </label>
      <label>
        User
        <input bind:value={user} placeholder="git" />
      </label>
      <label>
        Port
        <input bind:value={port} placeholder="22" />
      </label>
      <label>
        Identity File
        <select bind:value={identityFile}>
          <option value="">None</option>
          {#each keysStore.keys as key (key.name)}
            <option value={`~/.ssh/${key.name}`}>{key.name}</option>
          {/each}
        </select>
      </label>

      <div class="preview">
        {#if mode === "edit"}
          <div>
            <strong>Before</strong>
            <pre>{beforePreview}</pre>
          </div>
        {/if}
        <div>
          <strong>{mode === "edit" ? "After" : "Preview"}</strong>
          <pre>{afterPreview}</pre>
        </div>
      </div>

      {#if error}
        <p class="error">{error}</p>
      {/if}
      <div class="actions">
        <button type="button" onclick={onClose}>Cancel</button>
        <button type="submit" disabled={submitting}>
          {submitting ? "Saving..." : "Save"}
        </button>
      </div>
    </form>
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
    max-height: 90vh;
    overflow-y: auto;
  }
  label {
    display: block;
    margin-bottom: 0.75rem;
  }
  input,
  select {
    width: 100%;
    margin-top: 0.25rem;
  }
  .preview {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.75rem;
  }
  .preview > div {
    flex: 1;
  }
  .preview pre {
    background: #f6f6f6;
    padding: 0.5rem;
    font-size: 0.75rem;
    white-space: pre-wrap;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  .error {
    color: #c0392b;
  }
</style>
