<script lang="ts">
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { commands } from '$lib/tauri';

	const selected = $derived($selectedInstance ?? $instances[0] ?? null);

	async function toggleSync() {
		if (!selected) return;
		const newVal = !selected.syncOptions;
		if (newVal) {
			const ok = confirm(
				'Enabling sync will replace this instance\'s options.txt with the shared global settings. Continue?'
			);
			if (!ok) return;
		}
		try {
			await commands.setSyncOptions(selected.id, newVal);
			const updated = await commands.getInstance(selected.id);
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
		} catch (e) {
			alert(String(e));
		}
	}
</script>

<div class="flex flex-col gap-6 max-w-xl">
	<h1 class="text-xl font-semibold">Settings</h1>

	{#if selected}
		<section class="flex flex-col gap-3">
			<h2 class="text-sm font-medium" style="color: var(--text-secondary)">
				Instance — {selected.name}
			</h2>

			<div
				class="flex items-center justify-between p-4 rounded-lg border"
				style="background: var(--bg-surface); border-color: var(--border)"
			>
				<div class="flex flex-col gap-0.5">
					<span class="text-sm font-medium">Sync options.txt</span>
					<span class="text-xs" style="color: var(--text-secondary)">
						Share game settings across all instances with sync enabled.
					</span>
				</div>
				<button
					class="relative w-10 h-6 rounded-full transition-colors flex-shrink-0"
					style="background: {selected.syncOptions ? 'var(--accent)' : 'var(--bg-raised)'}"
					onclick={toggleSync}
					aria-label="Toggle sync"
				>
					<span
						class="absolute top-1 w-4 h-4 rounded-full bg-white transition-transform"
						style="transform: translateX({selected.syncOptions ? '18px' : '4px'})"
					></span>
				</button>
			</div>

			<div
				class="flex items-center justify-between p-4 rounded-lg border"
				style="background: var(--bg-surface); border-color: var(--border)"
			>
				<div>
					<span class="text-sm font-medium">Allocated RAM</span>
					<p class="text-xs" style="color: var(--text-secondary)">{selected.allocatedRamMb} MB</p>
				</div>
				<span class="text-xs" style="color: var(--text-muted)">Edit via instance.json</span>
			</div>
		</section>
	{:else}
		<p class="text-sm" style="color: var(--text-muted)">Select an instance on the Home page to configure it.</p>
	{/if}

	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium" style="color: var(--text-secondary)">About</h2>
		<div
			class="p-4 rounded-lg border"
			style="background: var(--bg-surface); border-color: var(--border)"
		>
			<p class="text-sm font-semibold">Cirrus</p>
			<p class="text-xs mt-1" style="color: var(--text-muted)">Version 0.1.0</p>
			<p class="text-xs mt-1" style="color: var(--text-muted)">Open-source Minecraft launcher</p>
		</div>
	</section>
</div>
