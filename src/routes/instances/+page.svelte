<script lang="ts">
	import { instances, instancesLoading } from '$lib/stores/instances';
	import { commands, type Loader } from '$lib/tauri';
	import InstanceCard from '$lib/components/InstanceCard.svelte';

	let showCreate = $state(false);
	let creating = $state(false);
	let createError = $state<string | null>(null);

	let name = $state('My Instance');
	let mcVersion = $state('1.21.4');
	let loader = $state<Loader>('fabric');
	let loaderVersion = $state('');
	let versionList = $state<string[]>([]);
	let loadingVersions = $state(false);

	async function loadVersions() {
		loadingVersions = true;
		try {
			versionList = await commands.getVersionList();
		} catch {
			versionList = [];
		} finally {
			loadingVersions = false;
		}
	}

	async function handleCreate() {
		creating = true;
		createError = null;
		try {
			const inst = await commands.createInstance(
				name,
				mcVersion,
				loader,
				loaderVersion || undefined
			);
			instances.update((list) => [inst, ...list]);
			showCreate = false;
			name = 'My Instance';
		} catch (e) {
			createError = String(e);
		} finally {
			creating = false;
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this instance? This cannot be undone.')) return;
		try {
			await commands.deleteInstance(id);
			instances.update((list) => list.filter((i) => i.id !== id));
		} catch (e) {
			alert(String(e));
		}
	}

	$effect(() => {
		if (showCreate && versionList.length === 0) {
			loadVersions();
		}
	});
</script>

<div class="flex flex-col gap-6">
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-semibold">Instances</h1>
		<button
			class="px-4 py-2 rounded-lg text-sm font-medium"
			style="background: var(--accent); color: #fff"
			onclick={() => (showCreate = !showCreate)}
		>
			{showCreate ? 'Cancel' : 'New Instance'}
		</button>
	</div>

	{#if showCreate}
		<div class="p-4 rounded-lg border flex flex-col gap-3" style="background: var(--bg-surface); border-color: var(--border)">
			<h2 class="font-semibold text-sm">Create Instance</h2>

			<label class="flex flex-col gap-1">
				<span class="text-xs" style="color: var(--text-secondary)">Name</span>
				<input
					bind:value={name}
					class="px-3 py-2 rounded text-sm"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
				/>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-xs" style="color: var(--text-secondary)">Minecraft Version</span>
				{#if versionList.length > 0}
					<select
						bind:value={mcVersion}
						class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
					>
						{#each versionList as v}
							<option value={v}>{v}</option>
						{/each}
					</select>
				{:else}
					<input
						bind:value={mcVersion}
						class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
						placeholder={loadingVersions ? 'Loading versions…' : '1.21.4'}
					/>
				{/if}
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-xs" style="color: var(--text-secondary)">Loader</span>
				<select
					bind:value={loader}
					class="px-3 py-2 rounded text-sm"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
				>
					<option value="vanilla">Vanilla</option>
					<option value="fabric">Fabric</option>
					<option value="forge">Forge</option>
					<option value="quilt">Quilt</option>
					<option value="neoforge">NeoForge</option>
				</select>
			</label>

			{#if loader !== 'vanilla'}
				<label class="flex flex-col gap-1">
					<span class="text-xs" style="color: var(--text-secondary)">Loader Version (optional)</span>
					<input
						bind:value={loaderVersion}
						class="px-3 py-2 rounded text-sm"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary)"
						placeholder="latest"
					/>
				</label>
			{/if}

			{#if createError}
				<p class="text-xs" style="color: var(--danger)">{createError}</p>
			{/if}

			<button
				class="px-4 py-2 rounded-lg text-sm font-medium self-end disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={handleCreate}
				disabled={creating || !name.trim()}
			>
				{creating ? 'Creating…' : 'Create'}
			</button>
		</div>
	{/if}

	{#if $instancesLoading}
		<p class="text-sm" style="color: var(--text-muted)">Loading…</p>
	{:else if $instances.length === 0}
		<div class="flex flex-col items-center justify-center gap-3 py-16">
			<span class="text-4xl">📦</span>
			<p class="text-sm" style="color: var(--text-muted)">No instances yet. Create one above.</p>
		</div>
	{:else}
		<div class="grid gap-2" style="grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))">
			{#each $instances as inst}
				<div class="relative group">
					<InstanceCard instance={inst} />
					<button
						class="absolute top-2 right-2 w-6 h-6 rounded text-xs opacity-0 group-hover:opacity-100 transition-opacity"
						style="background: var(--danger); color: #fff"
						onclick={() => handleDelete(inst.id)}
						title="Delete instance"
					>✕</button>
				</div>
			{/each}
		</div>
	{/if}
</div>
