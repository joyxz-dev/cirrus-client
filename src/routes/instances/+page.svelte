<script lang="ts">
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
	async function handleDelete(id: string) {
		if (!confirm('Delete this instance? All files in its directory will be removed.')) return;
		try {
			await commands.deleteInstance(id);
			instances.update((list) => list.filter((i) => i.id !== id));
			if (editingInst?.id === id) editingInst = null;
		} catch (e) {
			alert(String(e));
		}
	}

	// ── Edit ─────────────────────────────────────────────────────────────────────
	let editingInst = $state<Instance | null>(null);
	let editName = $state('');
	let editRam = $state(2048);
	let editWidth = $state(1920);
	let editHeight = $state(1080);
	let editSaving = $state(false);
	let editError = $state('');

	function startEdit(inst: Instance) {
		if (editingInst?.id === inst.id) { editingInst = null; return; }
		editingInst = inst;
		editName = inst.name;
		editRam = inst.allocatedRamMb;
		editWidth = inst.resolution.width;
		editHeight = inst.resolution.height;
		editError = '';
	}

	async function saveEdit() {
		if (!editingInst) return;
		editSaving = true;
		editError = '';
		try {
			const updated = await commands.updateInstance(editingInst.id, {
				name: editName,
				allocatedRamMb: editRam,
				resolutionWidth: editWidth,
				resolutionHeight: editHeight,
			});
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			if ($selectedInstance?.id === updated.id) selectedInstance.set(updated);
			editingInst = null;
		} catch (e) {
			editError = String(e);
		} finally {
			editSaving = false;
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

	<!-- Instance list -->
	{#if $instancesLoading}
		<p class="text-sm" style="color: var(--text-muted)">Loading…</p>
	{:else if $instances.length === 0}
		<div class="flex flex-col items-center justify-center gap-3 py-16">
			<p class="text-sm" style="color: var(--text-muted)">No instances yet.</p>
		</div>
	{:else}
		<div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))">
			{#each $instances as inst}
				<div class="relative group">
					<InstanceCard
						instance={inst}
						selected={editingInst?.id === inst.id}
					/>
					<!-- Edit button -->
					<button
						class="absolute top-2 right-9 w-6 h-6 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
						style="background: var(--bg-hover); color: var(--text-secondary)"
						onclick={() => startEdit(inst)}
						title="Edit instance"
					>✎</button>
					<!-- Delete button -->
					<button
						class="absolute top-2 right-2 w-6 h-6 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"
						style="background: var(--danger); color: #fff"
						onclick={() => handleDelete(inst.id)}
						title="Delete instance"
					>✕</button>
				</div>
			{/each}
		</div>

		<!-- Edit panel -->
		{#if editingInst}
			<div class="p-4 rounded-lg border flex flex-col gap-4" style="background: var(--bg-surface); border-color: var(--accent)">
				<h2 class="text-sm font-semibold">Edit — {editingInst.name}</h2>

				<label class="flex flex-col gap-1">
					<span class="text-xs" style="color: var(--text-secondary)">Name</span>
					<input
						bind:value={editName}
						class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
					/>
				</label>

				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<span class="text-xs" style="color: var(--text-secondary)">Allocated RAM</span>
						<div class="flex items-center gap-1.5">
							<input
								type="number"
								bind:value={editRam}
								min="512"
								max="65536"
								step="256"
								class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
								style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--accent); outline: none"
							/>
							<span class="text-xs" style="color: var(--text-muted)">MB</span>
						</div>
					</div>
					<input
						type="range"
						bind:value={editRam}
						min="512"
						max="16384"
						step="256"
						class="w-full"
						style="accent-color: var(--accent)"
					/>
					<div class="flex justify-between text-xs" style="color: var(--text-muted)">
						<span>512 MB</span>
						<span>16 GB</span>
					</div>
				</div>

				<div class="flex items-center justify-between gap-4">
					<span class="text-xs" style="color: var(--text-secondary)">Resolution</span>
					<div class="flex items-center gap-2">
						<input
							type="number"
							bind:value={editWidth}
							min="640"
							max="7680"
							class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
							style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
						/>
						<span class="text-xs" style="color: var(--text-muted)">×</span>
						<input
							type="number"
							bind:value={editHeight}
							min="480"
							max="4320"
							class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
							style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
						/>
						<span class="text-xs" style="color: var(--text-muted)">px</span>
					</div>
				</div>

				{#if editError}
					<p class="text-xs select-text cursor-text" style="color: var(--danger)">{editError}</p>
				{/if}

				<div class="flex gap-2 justify-end">
					<button
						class="px-4 py-1.5 rounded text-xs font-medium"
						style="background: var(--bg-raised); color: var(--text-secondary)"
						onclick={() => (editingInst = null)}
					>Cancel</button>
					<button
						class="px-4 py-1.5 rounded text-xs font-medium disabled:opacity-50"
						style="background: var(--accent); color: #fff"
						onclick={saveEdit}
						disabled={editSaving || !editName.trim()}
					>
						{editSaving ? 'Saving…' : 'Save'}
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>
