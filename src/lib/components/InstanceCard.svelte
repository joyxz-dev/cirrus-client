<script lang="ts">
	import type { Instance } from '$lib/tauri';

	let {
		instance,
		selected = false,
		onclick,
	}: {
		instance: Instance;
		selected?: boolean;
		onclick?: () => void;
	} = $props();

	const loaderColors: Record<string, string> = {
		vanilla: '#5cad5c',
		fabric: '#c6a94f',
		forge: '#4a8fc1',
		quilt: '#b060d0',
		neoforge: '#e08840',
	};

	const color = $derived(loaderColors[instance.loader] ?? '#888');
</script>

<button
	class="w-full flex items-center gap-3 p-3 rounded-lg border text-left transition-colors"
	style="
		background: {selected ? 'var(--bg-raised)' : 'var(--bg-surface)'};
		border-color: {selected ? 'var(--accent)' : 'var(--border)'};
		color: var(--text-primary);
	"
	{onclick}
>
	<!-- Icon placeholder -->
	<div
		class="w-10 h-10 rounded-lg flex items-center justify-center text-white font-bold flex-shrink-0"
		style="background: {color}; font-size: 18px"
	>
		{instance.name.charAt(0).toUpperCase()}
	</div>

	<div class="flex flex-col gap-0.5 min-w-0">
		<span class="font-medium text-sm truncate">{instance.name}</span>
		<span class="text-xs" style="color: var(--text-secondary)">
			{instance.mcVersion} · {instance.loader}
			{#if instance.loaderVersion}
				{instance.loaderVersion}
			{/if}
		</span>
		{#if instance.mods.length > 0}
			<span class="text-xs" style="color: var(--text-muted)">{instance.mods.length} mod{instance.mods.length !== 1 ? 's' : ''}</span>
		{/if}
	</div>
</button>
