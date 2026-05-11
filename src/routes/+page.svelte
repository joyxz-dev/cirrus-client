<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { account, authLoading, authError } from '$lib/stores/account';
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { launchState, launchProgress, launchMessage, launchError } from '$lib/stores/launch';
	import { commands, type Account, type DownloadProgress } from '$lib/tauri';
	import InstanceCard from '$lib/components/InstanceCard.svelte';
	import LaunchButton from '$lib/components/LaunchButton.svelte';
	import ProgressBar from '$lib/components/ProgressBar.svelte';

	let authChallenge: { user_code: string; verification_uri: string; message: string } | null = $state(null);
	let codeCopied = $state(false);
	const unlisteners: (() => void)[] = [];

	async function copyCode(code: string) {
		await navigator.clipboard.writeText(code);
		codeCopied = true;
		setTimeout(() => (codeCopied = false), 2000);
	}

	onMount(async () => {
		const { listen } = await import('@tauri-apps/api/event');

		unlisteners.push(await listen<Account>('auth:complete', ({ payload }) => {
			account.set(payload);
			authChallenge = null;
			authLoading.set(false);
		}));

		unlisteners.push(await listen<string>('auth:error', ({ payload }) => {
			authError.set(payload);
			authChallenge = null;
			authLoading.set(false);
		}));

		unlisteners.push(await listen<DownloadProgress>('download:progress', ({ payload }) => {
			const phaseLabels: Record<string, string> = {
				client: 'Downloading client…',
				assets: 'Downloading assets…',
				libraries: 'Downloading libraries…',
			};
			launchMessage.set(phaseLabels[payload.phase] ?? 'Downloading…');
			launchState.set('downloading');
			if (payload.total > 0) {
				launchProgress.set(Math.round((payload.current / payload.total) * 100));
			}
		}));
	});

	onDestroy(() => {
		unlisteners.forEach((u) => u());
	});

	async function handleLogin() {
		authLoading.set(true);
		authError.set(null);
		try {
			const challenge = await commands.startAuth();
			authChallenge = challenge;
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
		launchProgress.set(0);
		launchMessage.set('');
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
				class="px-4 py-2 rounded-lg text-sm font-medium disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={handleLogin}
				disabled={$authLoading}
			>
				{$authLoading ? 'Connecting…' : 'Sign in with Microsoft'}
			</button>
		{/if}
	</div>

	<!-- Auth challenge -->
	{#if authChallenge}
		<div class="p-4 rounded-lg border relative" style="background: var(--bg-surface); border-color: var(--border)">
			<p class="text-sm mb-2" style="color: var(--text-secondary)">
				Go to <strong style="color: var(--text-primary)">{authChallenge.verification_uri}</strong> and enter:
			</p>
			<button
				class="text-3xl font-mono font-bold tracking-widest cursor-pointer transition-opacity hover:opacity-70 active:opacity-50"
				style="color: var(--accent); background: none; border: none; padding: 0"
				onclick={() => copyCode(authChallenge!.user_code)}
				title="Click to copy"
			>
				{authChallenge.user_code}
			</button>
			<p class="text-xs mt-2" style="color: var(--text-muted)">Waiting for sign-in… · Click code to copy</p>

			{#if codeCopied}
				<div
					class="absolute bottom-3 right-3 text-xs px-2 py-1 rounded"
					style="background: var(--success); color: #fff"
				>
					Code copied!
				</div>
			{/if}
		</div>
	{/if}

	{#if $authError}
		<p class="text-sm select-text cursor-text" style="color: var(--danger)">Sign-in error: {$authError}</p>
	{/if}

	<!-- Instance grid -->
	<div class="flex-1 overflow-y-auto">
		{#if $instances.length === 0}
			<div class="flex flex-col items-center justify-center h-full gap-3">
				<p class="text-sm" style="color: var(--text-muted)">
					No instances yet. <a href="/instances" style="color: var(--accent)">Create one</a>
				</p>
			</div>
		{:else}
			<div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(220px, 1fr))">
				{#each $instances as inst}
					<InstanceCard
						instance={inst}
						selected={$selectedInstance?.id === inst.id}
						onclick={() => selectedInstance.set(inst)}
					/>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Launch bar -->
	{#if $selectedInstance}
		<div class="flex flex-col gap-2 pt-3 border-t" style="border-color: var(--border)">
			{#if $launchState === 'downloading'}
				<ProgressBar label={$launchMessage} value={$launchProgress} />
			{/if}

			{#if $launchError}
				<p class="text-xs select-text cursor-text" style="color: var(--danger)">{$launchError}</p>
			{/if}

			<div class="flex items-center justify-between">
				<div class="flex flex-col">
					<span class="text-sm font-medium">{$selectedInstance.name}</span>
					<span class="text-xs" style="color: var(--text-muted)">
						MC {$selectedInstance.mcVersion} · {$selectedInstance.loader}
						· {$selectedInstance.allocatedRamMb} MB
					</span>
				</div>
				{#if !$account}
					<p class="text-xs" style="color: var(--text-muted)">Sign in to launch</p>
				{:else}
					<LaunchButton
						state={$launchState}
						disabled={false}
						onclick={handleLaunch}
					/>
				{/if}
			</div>
		</div>
	{/if}
</div>
