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

<div class="mod-card flex items-center gap-3 p-3 rounded-xl border"
	style="background: var(--bg-surface); border-color: var(--border)">

	{#if m.iconUrl}
		<img src={m.iconUrl} alt={m.title} class="w-10 h-10 rounded-lg flex-shrink-0 object-cover" style="background: var(--bg-raised)" />
	{:else}
		<div class="w-10 h-10 rounded-lg flex-shrink-0 flex items-center justify-center" style="background: var(--bg-raised)">
			<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
				<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
				<polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
				<line x1="12" y1="22.08" x2="12" y2="12"/>
			</svg>
		</div>
	{/if}

	<div class="flex flex-col gap-0.5 flex-1 min-w-0">
		<div class="flex items-center gap-2">
			<span class="font-semibold text-sm truncate" style="color: var(--text-primary)">{m.title}</span>
			<span class="text-xs flex-shrink-0 flex items-center gap-0.5" style="color: var(--text-muted)">
				<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>
				</svg>
				{formatDownloads(m.downloads)}
			</span>
		</div>
		<p class="text-xs line-clamp-1" style="color: var(--text-secondary)">{m.description}</p>
	</div>

	<button
		class="install-btn flex-shrink-0 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all disabled:opacity-50"
		class:installed
		disabled={installed || installing}
		onclick={oninstall}
	>
		{#if installing}
			<span class="flex items-center gap-1">
				<span class="spinner"></span>
			</span>
		{:else if installed}
			Installed
		{:else}
			Install
		{/if}
	</button>
</div>

<style>
	.mod-card {
		transition: border-color 0.15s ease, box-shadow 0.15s ease;
	}
	.mod-card:hover {
		border-color: var(--border-strong);
		box-shadow: 0 2px 12px rgba(0,0,0,0.2);
	}

	.install-btn {
		background: var(--accent);
		color: #fff;
		box-shadow: 0 0 12px rgba(99, 102, 241, 0.3);
	}
	.install-btn:not(:disabled):hover {
		background: var(--accent-hover);
		box-shadow: 0 0 20px rgba(99, 102, 241, 0.5);
		transform: translateY(-1px);
	}
	.install-btn.installed {
		background: var(--bg-raised);
		color: var(--text-muted);
		box-shadow: none;
	}

	.spinner {
		display: inline-block;
		width: 10px;
		height: 10px;
		border: 2px solid rgba(255,255,255,0.3);
		border-top-color: #fff;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
