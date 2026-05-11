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
		vanilla: '#4ade80',
		fabric:  '#facc15',
		forge:   '#60a5fa',
		quilt:   '#c084fc',
		neoforge:'#fb923c',
	};

	const color = $derived(loaderColors[instance.loader] ?? '#8b95b0');
</script>

<button
	class="card w-full flex items-center gap-3 p-3 rounded-xl text-left"
	class:selected
	style="--loader-color: {color}"
	{onclick}
>
	<!-- Icon -->
	<div class="icon flex items-center justify-center w-9 h-9 rounded-lg flex-shrink-0 font-bold text-white"
		style="background: linear-gradient(135deg, {color}cc 0%, {color}66 100%); font-size: 16px; letter-spacing: 0; box-shadow: 0 2px 8px {color}44">
		{instance.name.charAt(0).toUpperCase()}
	</div>

	<div class="flex flex-col gap-0.5 min-w-0 flex-1">
		<span class="font-semibold text-sm truncate" style="color: var(--text-primary)">{instance.name}</span>
		<span class="text-xs truncate" style="color: var(--text-secondary)">
			{instance.mcVersion} · <span style="color: {color}">{instance.loader}</span>
			{#if instance.loaderVersion}&nbsp;{instance.loaderVersion}{/if}
		</span>
		<span class="text-xs" style="color: var(--text-muted)">{instance.mods.length} mod{instance.mods.length !== 1 ? 's' : ''}</span>
	</div>
</button>

<style>
	.card {
		background: var(--bg-surface);
		border: 1px solid var(--border);
		transition: background 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
	}
	.card:hover {
		background: var(--bg-raised);
		border-color: var(--border-strong);
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
		transform: translateY(-1px);
	}
	.card.selected {
		background: var(--bg-raised);
		border-color: var(--accent);
		box-shadow: 0 0 0 1px var(--accent-dim), 0 4px 20px rgba(0, 0, 0, 0.3);
	}
</style>
