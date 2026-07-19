<!-- src/lib/components/ui/Button.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    variant = 'secondary',
    type = 'button',
    disabled = false,
    onclick,
    children,
  }: {
    variant?: 'primary' | 'secondary' | 'danger';
    type?: 'button' | 'submit';
    disabled?: boolean;
    onclick?: (event: MouseEvent) => void;
    children: Snippet;
  } = $props();

  const variantClass: Record<string, string> = {
    primary: 'bg-text text-surface hover:bg-text/90',
    secondary:
      'bg-transparent text-text border border-border hover:bg-canvas',
    danger: 'bg-danger-bg text-danger hover:bg-danger hover:text-surface',
  };
</script>

<button
  {type}
  {disabled}
  {onclick}
  class="rounded-sm px-3 py-1.5 text-sm font-medium transition-colors duration-150 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50 {variantClass[
    variant
  ]}"
>
  {@render children()}
</button>
