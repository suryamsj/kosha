<!-- src/lib/components/ui/Badge.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    variant = 'info',
    children,
  }: {
    variant?: 'info' | 'success' | 'warning' | 'danger';
    children: Snippet;
  } = $props();

  const variantClass: Record<string, string> = {
    info: 'bg-info-bg text-info',
    success: 'bg-success-bg text-success',
    warning: 'bg-warning-bg text-warning',
    danger: 'bg-danger-bg text-danger',
  };
</script>

<span
  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium uppercase tracking-wide {variantClass[
    variant
  ]}"
>
  {@render children()}
</span>
