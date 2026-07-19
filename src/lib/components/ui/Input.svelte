<!-- src/lib/components/ui/Input.svelte -->
<script lang="ts">
  let {
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    label,
    error = null,
    id,
  }: {
    value?: string;
    type?: string;
    placeholder?: string;
    label?: string;
    error?: string | null;
    id?: string;
  } = $props();

  const inputId = $derived(id ?? label?.toLowerCase().replace(/\s+/g, '-'));
</script>

<div class="flex flex-col gap-1">
  {#if label}
    <label for={inputId} class="text-sm font-medium text-text">{label}</label>
  {/if}
  <input
    id={inputId}
    {type}
    {placeholder}
    bind:value
    class="rounded-sm border border-border bg-surface px-3 py-1.5 text-sm text-text placeholder:text-text-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-text"
  />
  {#if error}
    <p class="text-xs text-danger">{error}</p>
  {/if}
</div>
