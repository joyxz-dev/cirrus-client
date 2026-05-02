<script lang="ts">
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { commands, type ModResult, type ModSearchResult } from '$lib/tauri';
	import ModCard from '$lib/components/ModCard.svelte';

	let query = $state('');
	let results = $state<ModSearchResult | null>(null);
	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let installing = $state<Record<string, boolean>>({});

	const selected = $derived($selectedInstance ?? $instances[0] ?? null);

	async function handleSearch() {
		if (!query.trim()) return;
		searching = true;
		searchError = null;
		try {
			results = await commands.searchMods({
				query: query.trim(),
				loader: selected?.loader !== 'vanilla' ? selected?.loader : undefined,
				mcVersion: selected?.mcVersion,
				offset: 0,
				limit: 20,
			});
		} catch (e) {
			searchError = String(e);
		} finally {
			searching = false;
		}
	}

	async function handleInstall(mod: ModResult) {
		if (!selected) return;
		installing = { ...installing, [mod.id]: true };
		try {
			await commands.installMod(selected.id, mod.id);
			// Refresh the instance mods list
			const updated = await commands.getInstance(selected.id);
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
		} catch (e) {
			alert(String(e));
		} finally {
			installing = { ...installing, [mod.id]: false };
		}
	}

	function isInstalled(modId: string): boolean {
		return selected?.mods.some((m) => m.id === modId) ?? false;
	}
</script>

<div class="flex flex-col gap-6">
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-semibold">Mods</h1>
		{#if selected}
			<span class="text-xs px-2 py-1 rounded" style="background: var(--bg-surface); color: var(--text-secondary)">
				{selected.name}
			</span>
		{/if}
	</div>

	{#if !selected}
		<p class="text-sm" style="color: var(--text-muted)">Create an instance first to browse mods.</p>
	{:else}
		<div class="flex gap-2">
			<input
				bind:value={query}
				class="flex-1 px-3 py-2 rounded text-sm"
				style="background: var(--bg-surface); border: 1px solid var(--border); color: var(--text-primary)"
				placeholder="Search mods…"
				onkeydown={(e) => e.key === 'Enter' && handleSearch()}
			/>
			<button
				class="px-4 py-2 rounded text-sm font-medium disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={handleSearch}
				disabled={searching || !query.trim()}
			>
				{searching ? 'Searching…' : 'Search'}
			</button>
		</div>

		{#if searchError}
			<p class="text-xs" style="color: var(--danger)">{searchError}</p>
		{/if}

		{#if results}
			<div class="flex flex-col gap-2">
				{#if results.hits.length === 0}
					<p class="text-sm" style="color: var(--text-muted)">No mods found.</p>
				{:else}
					{#each results.hits as mod}
						<ModCard
							{mod}
							installed={isInstalled(mod.id)}
							installing={installing[mod.id] ?? false}
							oninstall={() => handleInstall(mod)}
						/>
					{/each}
				{/if}
			</div>
		{:else if !searching}
			<div class="flex flex-col items-center gap-3 py-16">
				<span class="text-4xl">🧩</span>
				<p class="text-sm" style="color: var(--text-muted)">Search for mods on Modrinth above.</p>
			</div>
		{/if}
	{/if}
</div>
