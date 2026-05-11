<script lang="ts">
	import { onMount } from 'svelte';
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { commands, type ModResult, type ModSearchResult } from '$lib/tauri';
	import ModCard from '$lib/components/ModCard.svelte';

	type SortIndex = 'downloads' | 'newest' | 'updated' | 'relevance';

	let query = $state('');
	let results = $state<ModSearchResult | null>(null);
	let searching = $state(false);
	let searchError = $state<string | null>(null);
	let installing = $state<Record<string, boolean>>({});
	let removing = $state<Record<string, boolean>>({});
	let tab = $state<'browse' | 'installed'>('browse');
	let sortIndex = $state<SortIndex>('downloads');
	let loaderFilter = $state('');
	let versionFilter = $state('');

	const LOADERS: string[] = ['', 'fabric', 'forge', 'quilt', 'neoforge'];
	const LOADER_LABELS: Record<string, string> = {
		'': 'All', fabric: 'Fabric', forge: 'Forge', quilt: 'Quilt', neoforge: 'NeoForge',
	};
	const SORTS: { value: SortIndex; label: string }[] = [
		{ value: 'downloads', label: 'Popular' },
		{ value: 'newest',    label: 'New' },
		{ value: 'updated',   label: 'Updated' },
		{ value: 'relevance', label: 'Relevant' },
	];

	const selected = $derived($selectedInstance ?? $instances[0] ?? null);
	const installedMods = $derived(selected?.mods ?? []);

	onMount(async () => {
		if (selected) {
			loaderFilter = selected.loader !== 'vanilla' ? selected.loader : '';
			versionFilter = selected.mcVersion;
		}
		await doSearch();
	});

	async function doSearch() {
		searching = true;
		searchError = null;
		try {
			results = await commands.searchMods({
				query: query.trim(),
				loader: loaderFilter || undefined,
				mcVersion: versionFilter || undefined,
				index: sortIndex,
				offset: 0,
				limit: 20,
			});
		} catch (e) {
			searchError = String(e);
		} finally {
			searching = false;
		}
	}

	function setSort(s: SortIndex) { sortIndex = s; doSearch(); }
	function setLoader(l: string)  { loaderFilter = l; doSearch(); }

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
			<!-- Search -->
			<div class="flex gap-2">
				<input
					bind:value={query}
					class="flex-1 px-3 py-2 rounded text-sm"
					style="background: var(--bg-surface); border: 1px solid var(--border); color: var(--text-primary); outline: none"
					placeholder="Search mods…"
					onkeydown={(e) => e.key === 'Enter' && doSearch()}
				/>
				<button
					class="px-4 py-2 rounded text-sm font-medium disabled:opacity-50"
					style="background: var(--accent); color: #fff"
					onclick={doSearch}
					disabled={searching}
				>
					{searching ? '…' : 'Search'}
				</button>
			</div>

			<!-- Filters -->
			<div class="flex flex-col gap-2 pb-1">
				<!-- Sort -->
				<div class="flex items-center gap-3">
					<span class="text-xs w-14 flex-shrink-0" style="color: var(--text-muted)">Sort</span>
					<div class="flex gap-1">
						{#each SORTS as sort}
							<button
								class="px-2.5 py-1 rounded text-xs font-medium transition-colors"
								style="background: {sortIndex === sort.value ? 'var(--accent)' : 'var(--bg-surface)'}; color: {sortIndex === sort.value ? '#fff' : 'var(--text-secondary)'}"
								onclick={() => setSort(sort.value)}
							>{sort.label}</button>
						{/each}
					</div>
				</div>

				<!-- Loader -->
				<div class="flex items-center gap-3">
					<span class="text-xs w-14 flex-shrink-0" style="color: var(--text-muted)">Loader</span>
					<div class="flex gap-1">
						{#each LOADERS as loader}
							<button
								class="px-2.5 py-1 rounded text-xs font-medium transition-colors"
								style="background: {loaderFilter === loader ? 'var(--accent)' : 'var(--bg-surface)'}; color: {loaderFilter === loader ? '#fff' : 'var(--text-secondary)'}"
								onclick={() => setLoader(loader)}
							>{LOADER_LABELS[loader]}</button>
						{/each}
					</div>
				</div>

				<!-- Version -->
				<div class="flex items-center gap-3">
					<span class="text-xs w-14 flex-shrink-0" style="color: var(--text-muted)">Version</span>
					<div class="flex items-center gap-1">
						<input
							bind:value={versionFilter}
							class="px-2.5 py-1 rounded text-xs"
							style="width: 6rem; background: var(--bg-surface); border: 1px solid var(--border); color: var(--text-primary); outline: none"
							placeholder="Any"
							onblur={doSearch}
							onkeydown={(e) => e.key === 'Enter' && doSearch()}
						/>
						{#if versionFilter}
							<button
								class="text-xs leading-none px-1"
								style="color: var(--text-muted)"
								onclick={() => { versionFilter = ''; doSearch(); }}
							>✕</button>
						{/if}
					</div>
				</div>
			</div>

			{#if searchError}
				<p class="text-xs select-text cursor-text" style="color: var(--danger)">{searchError}</p>
			{/if}

			<!-- Results -->
			{#if searching}
				<div class="flex items-center justify-center py-16">
					<p class="text-sm" style="color: var(--text-muted)">Loading…</p>
				</div>
			{:else if results}
				{#if results.hits.length === 0}
					<p class="text-sm py-4" style="color: var(--text-muted)">No mods found.</p>
				{:else}
					<div class="flex flex-col gap-2">
						{#each results.hits as mod}
							<ModCard
								{mod}
								installed={isInstalled(mod.id)}
								installing={installing[mod.id] ?? false}
								oninstall={() => handleInstall(mod)}
							/>
						{/each}
					</div>
				{/if}
			{/if}
		{:else}
			<!-- Installed mods -->
			{#if installedMods.length === 0}
				<div class="flex flex-col items-center gap-3 py-12">
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
