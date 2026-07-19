<script lang="ts">
	import './layout.css';
	import { themeStore } from '$lib/theme.svelte';

	const { children } = $props();

	themeStore.init();
</script>

{@render children()}
