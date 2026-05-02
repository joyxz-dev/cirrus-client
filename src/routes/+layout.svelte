<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { onMount } from 'svelte';
	import { commands } from '$lib/tauri';
	import { account } from '$lib/stores/account';
	import { instances } from '$lib/stores/instances';

	let { children } = $props();

	onMount(async () => {
		// Load account state
		try {
			const acc = await commands.getAccount();
			account.set(acc);
		} catch {
			account.set(null);
		}

		// Load instances
		try {
			const list = await commands.listInstances();
			instances.set(list);
		} catch {
			instances.set([]);
		}
	});
</script>

<div class="flex h-screen w-screen overflow-hidden" style="background: var(--bg-base)">
	<Sidebar />
	<main class="flex-1 overflow-auto p-6">
		{@render children()}
	</main>
</div>
