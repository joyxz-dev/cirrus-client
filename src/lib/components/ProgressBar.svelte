<script lang="ts">
	let { value = 0, label = '' }: { value: number; label?: string } = $props();
</script>

<div class="w-full">
	{#if label}
		<p class="text-xs mb-1" style="color: var(--text-secondary)">{label}</p>
	{/if}
	<div class="w-full h-1.5 rounded-full" style="background: var(--bg-raised)">
		<div
			class="h-1.5 rounded-full transition-all duration-300"
			style="width: {Math.min(100, Math.max(0, value))}%; background: var(--accent)"
		></div>
	</div>
</div>
