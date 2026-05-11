<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { instances } from '$lib/stores/instances';
	import { commands, type Instance } from '$lib/tauri';

	const id = $derived($page.params.id ?? '');

	let inst = $state<Instance | null>(null);
	let notFound = $state(false);

	$effect(() => {
		const found = $instances.find((i) => i.id === id) ?? null;
		if (found) {
			inst = found;
			notFound = false;
		} else {
			// Try loading from backend if store isn't populated yet
			commands.getInstance(id).then((i) => {
				inst = i;
			}).catch(() => {
				notFound = true;
			});
		}
	});

	// ── Tabs ──────────────────────────────────────────────────────────────────────
	type Tab = 'mods' | 'resourcepacks' | 'shaderpacks' | 'saves' | 'screenshots';
	const tabs: Tab[] = ['mods', 'resourcepacks', 'shaderpacks', 'saves', 'screenshots'];
	let activeTab = $state<Tab>('mods');
	let folderContents = $state<string[]>([]);
	let folderLoading = $state(false);

	async function switchTab(tab: Tab) {
		activeTab = tab;
		if (tab === 'mods') { folderContents = []; return; }
		folderLoading = true;
		try {
			folderContents = await commands.listInstanceFolder(id ?? '', tab);
		} catch {
			folderContents = [];
		} finally {
			folderLoading = false;
		}
	}

	async function removeMod(modId: string) {
		if (!inst) return;
		try {
			await commands.removeMod(inst.id, modId);
			const updated = { ...inst, mods: inst.mods.filter((m) => m.id !== modId) };
			inst = updated;
			instances.update((list) => list.map((i) => (i.id === id ? updated : i)));
		} catch (e) {
			alert(String(e));
		}
	}
</script>

<div class="flex flex-col h-full gap-0">
	<!-- Nav bar -->
	<div class="flex items-center gap-3 mb-6">
		<div class="flex items-center gap-1">
			<button
				class="w-8 h-8 rounded flex items-center justify-center text-base transition-colors"
				style="background: var(--bg-raised); color: var(--text-secondary)"
				onclick={() => history.back()}
				title="Back"
			>&#8592;</button>
			<button
				class="w-8 h-8 rounded flex items-center justify-center text-base transition-colors"
				style="background: var(--bg-raised); color: var(--text-secondary)"
				onclick={() => history.forward()}
				title="Forward"
			>&#8594;</button>
		</div>

		{#if inst}
			<div class="flex items-center gap-2 flex-1 min-w-0">
				<span class="text-xl font-semibold truncate">{inst.name}</span>
				<span class="text-xs px-2 py-0.5 rounded font-mono flex-shrink-0"
					style="background: var(--bg-raised); color: var(--text-muted)">
					MC {inst.mcVersion} · {inst.loader}
				</span>
			</div>
			<button
				class="text-xs px-3 py-1.5 rounded flex-shrink-0"
				style="background: var(--bg-raised); color: var(--text-secondary)"
				onclick={() => commands.openInstanceFolder(inst!.id)}
			>
				Open folder
			</button>
		{:else if notFound}
			<span class="text-xl font-semibold" style="color: var(--danger)">Instance not found</span>
		{:else}
			<span class="text-xl font-semibold" style="color: var(--text-muted)">Loading…</span>
		{/if}
	</div>

	{#if inst}
		<!-- Tabs -->
		<div class="flex gap-1 border-b mb-4" style="border-color: var(--border)">
			{#each tabs as tab}
				<button
					class="px-4 py-2 text-sm font-medium capitalize transition-colors"
					style="
						color: {activeTab === tab ? 'var(--text-primary)' : 'var(--text-muted)'};
						border-bottom: 2px solid {activeTab === tab ? 'var(--accent)' : 'transparent'};
						background: transparent;
						margin-bottom: -1px;
					"
					onclick={() => switchTab(tab)}
				>
					{tab}
				</button>
			{/each}
		</div>

		<!-- Tab content -->
		{#if activeTab === 'mods'}
			{#if inst.mods.length === 0}
				<div class="flex flex-col items-center justify-center py-16 gap-2">
					<p class="text-sm" style="color: var(--text-muted)">No mods installed.</p>
					<a href="/mods" class="text-xs" style="color: var(--accent)">Browse mods →</a>
				</div>
			{:else}
				<div class="flex flex-col gap-2">
					{#each inst.mods as mod}
						<div class="flex items-center justify-between px-4 py-3 rounded-lg border"
							style="background: var(--bg-surface); border-color: var(--border)">
							<div class="flex flex-col gap-0.5">
								<span class="text-sm font-medium">{mod.name}</span>
								<span class="text-xs font-mono" style="color: var(--text-muted)">{mod.version}</span>
							</div>
							<button
								class="text-xs px-3 py-1.5 rounded"
								style="background: var(--danger); color: #fff"
								onclick={() => removeMod(mod.id)}
							>Remove</button>
						</div>
					{/each}
				</div>
			{/if}
		{:else if folderLoading}
			<p class="text-sm" style="color: var(--text-muted)">Loading…</p>
		{:else if folderContents.length === 0}
			<div class="flex flex-col items-center justify-center py-16">
				<p class="text-sm" style="color: var(--text-muted)">No files in this folder.</p>
			</div>
		{:else}
			<div class="flex flex-col gap-1.5">
				{#each folderContents as file}
					<div class="px-4 py-2.5 rounded-lg border text-sm font-mono"
						style="background: var(--bg-surface); border-color: var(--border); color: var(--text-secondary)">
						{file}
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>
