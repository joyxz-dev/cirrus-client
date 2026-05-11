<script lang="ts">
	import { goto } from '$app/navigation';
	import { instances, instancesLoading, selectedInstance } from '$lib/stores/instances';
	import { commands, type Instance, type Loader } from '$lib/tauri';
	import InstanceCard from '$lib/components/InstanceCard.svelte';

	// ── Create ───────────────────────────────────────────────────────────────────
	let showCreate = $state(false);
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let name = $state('My Instance');
	let mcVersion = $state('');
	let loader = $state<Loader>('fabric');
	let loaderVersion = $state('');
	let showSnapshots = $state(false);
	let allVersions = $state<{ id: string; kind: string }[]>([]);
	let loadingVersions = $state(false);

	const visibleVersions = $derived(
		showSnapshots ? allVersions : allVersions.filter((v) => v.kind === 'release')
	);

	$effect(() => {
		if (showCreate && allVersions.length === 0) loadVersions();
	});
	$effect(() => {
		if (visibleVersions.length > 0 && !mcVersion) mcVersion = visibleVersions[0].id;
	});

	async function loadVersions() {
		loadingVersions = true;
		try { allVersions = await commands.getVersionList(); }
		catch { allVersions = []; }
		finally { loadingVersions = false; }
	}

	async function handleCreate() {
		creating = true;
		createError = null;
		try {
			const inst = await commands.createInstance(name, mcVersion, loader, loaderVersion || undefined);
			instances.update((list) => [inst, ...list]);
			showCreate = false;
			resetForm();
		} catch (e) {
			createError = String(e);
		} finally {
			creating = false;
		}
	}

	function resetForm() {
		name = 'My Instance';
		mcVersion = visibleVersions[0]?.id ?? '';
		loader = 'fabric';
		loaderVersion = '';
	}

	// ── Delete ───────────────────────────────────────────────────────────────────
	async function handleDelete(e: MouseEvent, id: string) {
		e.stopPropagation();
		if (!confirm('Delete this instance? All files in its directory will be removed.')) return;
		try {
			await commands.deleteInstance(id);
			instances.update((list) => list.filter((i) => i.id !== id));
			if ($selectedInstance?.id === id) selectedInstance.set(null);
		} catch (e) {
			alert(String(e));
		}
	}

	// ── Selection & navigation ────────────────────────────────────────────────────
	let clickTimer: ReturnType<typeof setTimeout> | null = null;

	function handleCardClick(inst: Instance) {
		if (clickTimer) {
			clearTimeout(clickTimer);
			clickTimer = null;
			selectedInstance.set(inst);
			goto(`/instances/${inst.id}`);
		} else {
			clickTimer = setTimeout(() => {
				clickTimer = null;
				selectedInstance.set(inst);
			}, 220);
		}
	}
</script>

<div class="flex flex-col gap-6">
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-semibold">Instances</h1>
		<button
			class="px-4 py-2 rounded-lg text-sm font-medium"
			style="background: var(--accent); color: #fff"
			onclick={() => { showCreate = !showCreate; if (!showCreate) createError = null; }}
		>
			{showCreate ? 'Cancel' : 'New Instance'}
		</button>
	</div>

	<!-- Create form -->
	{#if showCreate}
		<div class="p-4 rounded-lg border flex flex-col gap-3" style="background: var(--bg-surface); border-color: var(--border)">
			<h2 class="font-semibold text-sm">Create Instance</h2>

			<label class="flex flex-col gap-1">
				<span class="text-xs" style="color: var(--text-secondary)">Name</span>
				<input bind:value={name} class="px-3 py-2 rounded text-sm"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none" />
			</label>

			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<span class="text-xs" style="color: var(--text-secondary)">Minecraft Version</span>
					<label class="flex items-center gap-1.5 cursor-pointer">
						<input type="checkbox" bind:checked={showSnapshots} class="w-3 h-3" />
						<span class="text-xs" style="color: var(--text-muted)">Show snapshots</span>
					</label>
				</div>
				{#if loadingVersions}
					<div class="px-3 py-2 rounded text-sm" style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-muted)">Loading versions…</div>
				{:else if visibleVersions.length > 0}
					<select bind:value={mcVersion} class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)">
						{#each visibleVersions as v}
							<option value={v.id}>{v.id}</option>
						{/each}
					</select>
				{:else}
					<input bind:value={mcVersion} class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
						placeholder="e.g. 1.21.4" />
				{/if}
			</div>

			<label class="flex flex-col gap-1">
				<span class="text-xs" style="color: var(--text-secondary)">Loader</span>
				<select bind:value={loader} class="px-3 py-2 rounded text-sm"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)">
					<option value="vanilla">Vanilla</option>
					<option value="fabric">Fabric</option>
					<option value="forge">Forge</option>
					<option value="quilt">Quilt</option>
					<option value="neoforge">NeoForge</option>
				</select>
			</label>

			{#if loader !== 'vanilla'}
				<label class="flex flex-col gap-1">
					<span class="text-xs" style="color: var(--text-secondary)">
						Loader Version <span style="color: var(--text-muted)">(optional, leave blank for latest)</span>
					</span>
					<input bind:value={loaderVersion} class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
						placeholder="latest" />
				</label>
			{/if}

			{#if createError}
				<p class="text-xs" style="color: var(--danger)">{createError}</p>
			{/if}

			<button
				class="px-4 py-2 rounded-lg text-sm font-medium self-end disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={handleCreate}
				disabled={creating || !name.trim() || !mcVersion.trim()}
			>
				{creating ? 'Creating…' : 'Create'}
			</button>
		</div>
	{/if}

	<!-- Instance grid -->
	{#if $instancesLoading}
		<p class="text-sm" style="color: var(--text-muted)">Loading…</p>
	{:else if $instances.length === 0}
		<div class="flex flex-col items-center justify-center gap-3 py-16">
			<p class="text-sm" style="color: var(--text-muted)">No instances yet.</p>
		</div>
	{:else}
		<p class="text-xs" style="color: var(--text-muted)">Click to select · Double-click to open</p>
		<div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))">
			{#each $instances as inst}
				<div class="relative group">
					<InstanceCard
						instance={inst}
						selected={$selectedInstance?.id === inst.id}
						onclick={() => handleCardClick(inst)}
					/>
					<button
						class="absolute top-2 right-2 w-6 h-6 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
						style="background: var(--danger); color: #fff"
						onclick={(e) => handleDelete(e, inst.id)}
						title="Delete instance"
					>✕</button>
				</div>
			{/each}
		</div>
	{/if}
</div>
