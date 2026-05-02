<script lang="ts">
	import type { ModResult } from '$lib/tauri';

	let {
		mod: m,
		installing = false,
		installed = false,
		oninstall,
	}: {
		mod: ModResult;
		installing?: boolean;
		installed?: boolean;
		oninstall?: () => void;
	} = $props();

	function formatDownloads(n: number): string {
		if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
		if (n >= 1_000) return (n / 1_000).toFixed(0) + 'K';
		return String(n);
	}
</script>

<div
	class="flex items-start gap-3 p-3 rounded-lg border"
	style="background: var(--bg-surface); border-color: var(--border)"
>
	{#if m.iconUrl}
		<img src={m.iconUrl} alt={m.title} class="w-10 h-10 rounded-lg flex-shrink-0 object-cover" />
	{:else}
		<div
			class="w-10 h-10 rounded-lg flex-shrink-0 flex items-center justify-center text-lg"
			style="background: var(--bg-raised)"
		>🧩</div>
	{/if}

	<div class="flex flex-col gap-1 flex-1 min-w-0">
		<div class="flex items-baseline gap-2">
			<span class="font-medium text-sm">{m.title}</span>
			<span class="text-xs" style="color: var(--text-muted)">↓ {formatDownloads(m.downloads)}</span>
		</div>
		<p class="text-xs line-clamp-2" style="color: var(--text-secondary)">{m.description}</p>
	</div>

	<button
		class="flex-shrink-0 px-3 py-1.5 rounded text-xs font-medium transition-colors disabled:opacity-50"
		style="
			background: {installed ? 'var(--bg-raised)' : 'var(--accent)'};
			color: {installed ? 'var(--text-secondary)' : '#fff'};
		"
		disabled={installed || installing}
		onclick={oninstall}
	>
		{#if installing}
			…
		{:else if installed}
			Installed
		{:else}
			Install
		{/if}
	</button>
</div>
