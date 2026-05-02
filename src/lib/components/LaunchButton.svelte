<script lang="ts">
	import type { LaunchState } from '$lib/stores/launch';

	let {
		state = 'idle',
		disabled = false,
		onclick,
	}: {
		state?: LaunchState;
		disabled?: boolean;
		onclick?: () => void;
	} = $props();

	const labels: Record<LaunchState, string> = {
		idle: 'Launch',
		downloading: 'Downloading…',
		launching: 'Launching…',
		running: 'Running',
		error: 'Retry',
	};

	const bg: Record<LaunchState, string> = {
		idle: 'var(--accent)',
		downloading: 'var(--bg-raised)',
		launching: 'var(--bg-raised)',
		running: 'var(--success)',
		error: 'var(--danger)',
	};
</script>

<button
	class="px-8 py-3 rounded-lg font-semibold text-sm transition-all disabled:opacity-40"
	style="background: {bg[state]}; color: #fff; min-width: 140px"
	disabled={disabled || state === 'running' || state === 'downloading' || state === 'launching'}
	{onclick}
>
	{labels[state]}
</button>
