<script lang="ts">
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { commands, type ModResult, type ModSearchResult } from '$lib/tauri';
	import ModCard from '$lib/components/ModCard.svelte';

	let query = $state('');
	let results = $state<ModSearchResult | null>(null);
	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let installing = $state<Record<string, boolean>>({});
	let removing = $state<Record<string, boolean>>({});
	let tab = $state<'browse' | 'installed'>('browse');

	const selected = $derived($selectedInstance ?? $instances[0] ?? null);
	const installedMods = $derived(selected?.mods ?? []);

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
			const updated = await commands.getInstance(selected.id);
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
		} catch (e) {
			alert(String(e));
		} finally {
			installing = { ...installing, [mod.id]: false };
		}
	}

	async function handleRemove(modId: string) {
		if (!selected) return;
		removing = { ...removing, [modId]: true };
		try {
			await commands.removeMod(selected.id, modId);
			const updated = await commands.getInstance(selected.id);
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
		} catch (e) {
			alert(String(e));
		} finally {
			removing = { ...removing, [modId]: false };
		}
	}

	function isInstalled(modId: string): boolean {
		return installedMods.some((m) => m.id === modId);
	}
</script>

<div class="flex flex-col gap-4">
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
		<!-- Tabs -->
		<div class="flex gap-1 p-1 rounded-lg w-fit" style="background: var(--bg-surface)">
			{#each [['browse', 'Browse'], ['installed', `Installed (${installedMods.length})`]] as [id, label]}
				<button
					class="px-3 py-1.5 rounded text-xs font-medium transition-colors"
					style="background: {tab === id ? 'var(--bg-raised)' : 'transparent'}; color: {tab === id ? 'var(--text-primary)' : 'var(--text-secondary)'}"
					onclick={() => (tab = id as 'browse' | 'installed')}
				>
					{label}
				</button>
			{/each}
		</div>

		{#if tab === 'browse'}
			<div class="flex gap-2">
				<input
					bind:value={query}
					class="flex-1 px-3 py-2 rounded text-sm"
					style="background: var(--bg-surface); border: 1px solid var(--border); color: var(--text-primary); outline: none"
					placeholder="Search mods on Modrinth…"
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
				<div class="flex flex-col items-center gap-3 py-12">
					<span class="text-4xl">🧩</span>
					<p class="text-sm" style="color: var(--text-muted)">Search for mods above.</p>
				</div>
			{/if}
		{:else}
			<!-- Installed mods -->
			{#if installedMods.length === 0}
				<div class="flex flex-col items-center gap-3 py-12">
					<span class="text-4xl">📭</span>
					<p class="text-sm" style="color: var(--text-muted)">No mods installed yet.</p>
				</div>
			{:else}
				<div class="flex flex-col gap-2">
					{#each installedMods as mod}
						<div class="flex items-center gap-3 p-3 rounded-lg border" style="background: var(--bg-surface); border-color: var(--border)">
							<div class="flex-1 min-w-0">
								<p class="text-sm font-medium truncate">{mod.name}</p>
								<p class="text-xs" style="color: var(--text-muted)">{mod.version} · {mod.filename}</p>
							</div>
							<button
								class="flex-shrink-0 px-3 py-1.5 rounded text-xs font-medium disabled:opacity-50"
								style="background: var(--danger); color: #fff"
								disabled={removing[mod.id] ?? false}
								onclick={() => handleRemove(mod.id)}
							>
								{removing[mod.id] ? '…' : 'Remove'}
							</button>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	{/if}
</div>
