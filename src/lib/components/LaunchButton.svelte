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
		idle:        'Launch',
		downloading: 'Downloading…',
		launching:   'Launching…',
		running:     'Running',
		error:       'Retry',
	};

	const isDisabled = $derived(
		disabled || state === 'running' || state === 'downloading' || state === 'launching'
	);
</script>

<button
	class="launch-btn"
	class:idle={state === 'idle'}
	class:running={state === 'running'}
	class:busy={state === 'downloading' || state === 'launching'}
	class:error={state === 'error'}
	disabled={isDisabled}
	{onclick}
>
	{labels[state]}
</button>

<style>
	.launch-btn {
		padding: 10px 32px;
		border-radius: 10px;
		font-weight: 600;
		font-size: 13px;
		min-width: 140px;
		color: #fff;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
		position: relative;
		letter-spacing: 0.02em;
	}
	.launch-btn:disabled {
		cursor: not-allowed;
		opacity: 0.55;
	}

	.idle {
		background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
		box-shadow: 0 0 24px rgba(99, 102, 241, 0.4), 0 2px 8px rgba(0,0,0,0.3);
	}
	.idle:not(:disabled):hover {
		box-shadow: 0 0 32px rgba(99, 102, 241, 0.6), 0 4px 16px rgba(0,0,0,0.4);
		transform: translateY(-1px);
	}
	.idle:not(:disabled):active {
		transform: translateY(0);
		box-shadow: 0 0 16px rgba(99, 102, 241, 0.3);
	}

	.running {
		background: linear-gradient(135deg, #16a34a 0%, #22c55e 100%);
		box-shadow: 0 0 20px rgba(34, 197, 94, 0.35);
	}

	.busy {
		background: var(--bg-raised);
		box-shadow: none;
	}

	.error {
		background: linear-gradient(135deg, #be123c 0%, #f43f5e 100%);
		box-shadow: 0 0 20px rgba(244, 63, 94, 0.35);
	}
	.error:not(:disabled):hover {
		box-shadow: 0 0 28px rgba(244, 63, 94, 0.5);
		transform: translateY(-1px);
	}
</style>
