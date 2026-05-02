<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { account, authLoading, authError } from '$lib/stores/account';
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { launchState, launchMessage, launchError } from '$lib/stores/launch';
	import { commands, type Account } from '$lib/tauri';
	import InstanceCard from '$lib/components/InstanceCard.svelte';
	import LaunchButton from '$lib/components/LaunchButton.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';

	let authChallenge: { user_code: string; verification_uri: string; message: string } | null = $state(null);
	let unlistenComplete: (() => void) | null = null;
	let unlistenError: (() => void) | null = null;

	onMount(async () => {
		// Listen for auth events from the background polling task
		const { listen } = await import('@tauri-apps/api/event');
		unlistenComplete = await listen<Account>('auth:complete', ({ payload }) => {
			account.set(payload);
			authChallenge = null;
			authLoading.set(false);
		});
		unlistenError = await listen<string>('auth:error', ({ payload }) => {
			authError.set(payload);
			authChallenge = null;
			authLoading.set(false);
		});
	});

	onDestroy(() => {
		unlistenComplete?.();
		unlistenError?.();
	});

	async function handleLogin() {
		authLoading.set(true);
		authError.set(null);
		try {
			const challenge = await commands.startAuth();
			authChallenge = challenge;
			// authLoading stays true until auth:complete or auth:error
		} catch (e) {
			authError.set(String(e));
			authLoading.set(false);
		}
	}

	async function handleLogout() {
		await commands.logout();
		account.set(null);
	}

	async function handleLaunch() {
		const inst = $selectedInstance;
		if (!inst) return;

		launchState.set('launching');
		launchError.set(null);
		try {
			await commands.launchInstance(inst.id);
			launchState.set('idle');
		} catch (e) {
			launchState.set('error');
			launchError.set(String(e));
		}
	}
</script>

<div class="flex flex-col gap-6 h-full">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-xl font-semibold" style="color: var(--text-primary)">Home</h1>

		{#if $account}
			<div class="flex items-center gap-3">
				<div class="flex flex-col items-end">
					<span class="text-sm font-medium">{$account.username}</span>
					<span class="text-xs" style="color: var(--text-muted)">{$account.uuid.slice(0, 8)}…</span>
				</div>
				<button
					class="text-xs px-3 py-1.5 rounded transition-colors"
					style="background: var(--bg-raised); color: var(--text-secondary)"
					onclick={handleLogout}
				>
					Sign out
				</button>
			</div>
		{:else}
			<button
				class="px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={handleLogin}
				disabled={$authLoading}
			>
				{$authLoading ? 'Connecting…' : 'Sign in with Microsoft'}
			</button>
		{/if}
	</div>

	<!-- Auth challenge display -->
	{#if authChallenge}
		<div class="p-4 rounded-lg border" style="background: var(--bg-surface); border-color: var(--border)">
			<p class="text-sm mb-2" style="color: var(--text-secondary)">
				Go to <strong style="color: var(--text-primary)">{authChallenge.verification_uri}</strong> and enter:
			</p>
			<div class="text-3xl font-mono font-bold tracking-widest" style="color: var(--accent)">
				{authChallenge.user_code}
			</div>
			<p class="text-xs mt-2" style="color: var(--text-muted)">Waiting for sign-in…</p>
		</div>
	{/if}

	{#if $authError}
		<p class="text-sm" style="color: var(--danger)">Error: {$authError}</p>
	{/if}

	<!-- Main content -->
	<div class="flex gap-6 flex-1 min-h-0">
		<!-- Instance list -->
		<div class="flex flex-col gap-2 w-72 overflow-y-auto">
			<div class="flex items-center justify-between mb-1">
				<span class="text-xs font-medium uppercase tracking-wider" style="color: var(--text-muted)">Instances</span>
				<a href="/instances" class="text-xs" style="color: var(--accent)">Manage</a>
			</div>
			{#if $instances.length === 0}
				<p class="text-sm" style="color: var(--text-muted)">No instances yet. <a href="/instances" style="color: var(--accent)">Create one</a></p>
			{:else}
				{#each $instances as inst}
					<InstanceCard
						instance={inst}
						selected={$selectedInstance?.id === inst.id}
						onclick={() => selectedInstance.set(inst)}
					/>
				{/each}
			{/if}
		</div>

		<!-- Launch panel -->
		<div class="flex-1 flex flex-col justify-end gap-4">
			{#if $selectedInstance}
				<div class="p-4 rounded-lg border" style="background: var(--bg-surface); border-color: var(--border)">
					<h2 class="font-semibold mb-1">{$selectedInstance.name}</h2>
					<p class="text-xs" style="color: var(--text-secondary)">
						Minecraft {$selectedInstance.mcVersion} · {$selectedInstance.loader}
						{#if $selectedInstance.loaderVersion}&nbsp;{$selectedInstance.loaderVersion}{/if}
					</p>
					<p class="text-xs mt-1" style="color: var(--text-muted)">
						{$selectedInstance.allocatedRamMb} MB RAM · {$selectedInstance.mods.length} mods
					</p>
				</div>

				{#if $launchState === 'downloading'}
					<ProgressBar label={$launchMessage} value={50} />
				{/if}

				{#if $launchError}
					<p class="text-xs" style="color: var(--danger)">{$launchError}</p>
				{/if}

				<div class="flex justify-end">
					<LaunchButton
						state={$launchState}
						disabled={!$account}
						onclick={handleLaunch}
					/>
				</div>

				{#if !$account}
					<p class="text-xs text-right" style="color: var(--text-muted)">Sign in to launch</p>
				{/if}
			{:else}
				<div class="flex-1 flex items-center justify-center">
					<p class="text-sm" style="color: var(--text-muted)">Select an instance to launch</p>
				</div>
			{/if}
		</div>
	</div>
</div>
