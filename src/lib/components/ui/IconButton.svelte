<!-- src/lib/components/ui/IconButton.svelte -->
<script lang="ts">
  import type { Component } from 'svelte';

  let {
    icon: Icon,
    label,
    variant = 'secondary',
    onclick,
  }: {
    icon: Component<{ size?: number | string; weight?: 'bold' | 'duotone' | 'fill' | 'light' | 'thin' | 'regular' }>;
    label: string;
    variant?: 'primary' | 'secondary' | 'danger';
    onclick?: (event: MouseEvent) => void;
  } = $props();

  const variantClass: Record<string, string> = {
    primary: 'bg-text text-surface hover:bg-text/90',
    secondary:
      'bg-transparent text-text-muted border border-border hover:bg-canvas hover:text-text',
    danger: 'bg-transparent text-danger border border-border hover:bg-danger-bg',
  };
</script>

<button
  type="button"
  {onclick}
  aria-label={label}
  class="inline-flex h-8 w-8 items-center justify-center rounded-sm transition-colors duration-150 active:scale-[0.98] {variantClass[
    variant
  ]}"
>
  <Icon size={16} weight="bold" />
</button>
