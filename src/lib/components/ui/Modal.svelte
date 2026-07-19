<script lang="ts">
  import type { Snippet } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  let {
    title,
    onClose,
    width = '400px',
    children,
    footer,
  }: {
    title: string;
    onClose: () => void;
    width?: string;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onClose();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') onClose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div
  class="fixed inset-0 flex items-center justify-center bg-black/40"
  onclick={onBackdropClick}
  role="presentation"
  transition:fade={{ duration: 150 }}
>
  <div
    class="max-h-[90vh] overflow-y-auto rounded-md border border-border bg-surface p-6 shadow-sm"
    style:min-width={width}
    role="dialog"
    aria-modal="true"
    aria-label={title}
    transition:scale={{ duration: 150, start: 0.97 }}
  >
    <h2 class="mb-4 text-base font-semibold text-text">{title}</h2>
    {@render children()}
    {#if footer}
      <div class="mt-4 flex justify-end gap-2">
        {@render footer()}
      </div>
    {/if}
  </div>
</div>
