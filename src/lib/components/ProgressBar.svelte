<script lang="ts">
	let { value = 0, label = '' }: { value: number; label?: string } = $props();
	const pct = $derived(Math.min(100, Math.max(0, value)));
</script>

<div class="w-full flex flex-col gap-1.5">
	{#if label}
		<div class="flex items-center justify-between">
			<p class="text-xs" style="color: var(--text-secondary)">{label}</p>
			<p class="text-xs font-mono" style="color: var(--text-muted)">{pct}%</p>
		</div>
	{/if}
	<div class="w-full h-1 rounded-full overflow-hidden" style="background: var(--bg-raised)">
		<div
			class="h-full rounded-full"
			style="width: {pct}%; background: linear-gradient(90deg, var(--accent) 0%, #a855f7 100%); transition: width 0.3s ease; box-shadow: 0 0 8px rgba(99,102,241,0.6)"
		></div>
	</div>
</div>
