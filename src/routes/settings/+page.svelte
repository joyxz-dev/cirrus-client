<script lang="ts">
	import { onMount } from 'svelte';
	import { commands } from '$lib/tauri';

	// Azure client ID
	let clientId = $state('');
	let clientIdSaved = $state(false);
	let clientIdSaving = $state(false);
	let clientIdError = $state('');
	let showClientIdInput = $state(false);

	// Global defaults for new instances
	let ramMb = $state(2048);
	let resWidth = $state(1920);
	let resHeight = $state(1080);
	let defaultsSaving = $state(false);
	let defaultsSaved = $state(false);
	let defaultsError = $state('');

	onMount(async () => {
		try {
			const stored = await commands.getClientId();
			if (stored) clientId = stored;
		} catch {}

		try {
			const [ram, w, h] = await commands.getDefaults();
			ramMb = ram;
			resWidth = w;
			resHeight = h;
		} catch {}
	});

	async function saveClientId() {
		clientIdSaving = true;
		clientIdError = '';
		clientIdSaved = false;
		try {
			await commands.setClientId(clientId.trim());
			clientIdSaved = true;
			showClientIdInput = false;
			setTimeout(() => (clientIdSaved = false), 3000);
		} catch (e) {
			clientIdError = String(e);
		} finally {
			clientIdSaving = false;
		}
	}

	async function saveDefaults() {
		defaultsSaving = true;
		defaultsSaved = false;
		defaultsError = '';
		try {
			await commands.setDefaults(ramMb, resWidth, resHeight);
			defaultsSaved = true;
			setTimeout(() => (defaultsSaved = false), 3000);
		} catch (e) {
			defaultsError = String(e);
		} finally {
			defaultsSaving = false;
		}
	}

	const maskedId = $derived(
		clientId.length > 8 ? clientId.slice(0, 4) + '…' + clientId.slice(-4) : clientId
	);
</script>

<div class="flex flex-col gap-6 max-w-xl">
	<h1 class="text-xl font-semibold">Settings</h1>

	<!-- Microsoft Auth -->
	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium" style="color: var(--text-secondary)">Microsoft Authentication</h2>
		<div class="p-4 rounded-lg border flex flex-col gap-3" style="background: var(--bg-surface); border-color: var(--border)">
			<div class="flex items-start justify-between gap-4">
				<div class="flex flex-col gap-1">
					<span class="text-sm font-medium">Azure App Client ID</span>
					{#if clientId}
						<span class="text-xs font-mono" style="color: var(--accent)">{maskedId}</span>
					{:else}
						<span class="text-xs" style="color: var(--danger)">Not configured — sign-in will fail</span>
					{/if}
				</div>
				<button
					class="text-xs px-3 py-1.5 rounded flex-shrink-0"
					style="background: var(--bg-raised); color: var(--text-secondary)"
					onclick={() => (showClientIdInput = !showClientIdInput)}
				>
					{showClientIdInput ? 'Cancel' : clientId ? 'Change' : 'Set up'}
				</button>
			</div>

			{#if showClientIdInput}
				<div class="flex flex-col gap-2">
					<input
						bind:value={clientId}
						class="px-3 py-2 rounded text-sm font-mono"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
						placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
						onkeydown={(e) => e.key === 'Enter' && saveClientId()}
					/>
					{#if clientIdError}
						<p class="text-xs" style="color: var(--danger)">{clientIdError}</p>
					{/if}
					<button
						class="self-end px-4 py-1.5 rounded text-xs font-medium disabled:opacity-50"
						style="background: var(--accent); color: #fff"
						onclick={saveClientId}
						disabled={clientIdSaving || !clientId.trim()}
					>
						{clientIdSaving ? 'Saving…' : 'Save'}
					</button>
				</div>
			{/if}

			{#if clientIdSaved}
				<p class="text-xs" style="color: var(--success)">Saved. Sign in will use the new client ID.</p>
			{/if}

			<details>
				<summary class="text-xs cursor-pointer select-none" style="color: var(--text-muted)">
					How to get a client ID ▸
				</summary>
				<ol class="mt-2 flex flex-col gap-1.5 text-xs" style="color: var(--text-secondary)">
					<li>1. Go to <span class="font-mono" style="color: var(--accent)">portal.azure.com</span> and sign in.</li>
					<li>2. Open <strong>Azure Active Directory → App registrations → New registration</strong>.</li>
					<li>3. Name it anything, select <strong>Personal Microsoft accounts only</strong>.</li>
					<li>4. Leave redirect URI blank. Click <strong>Register</strong>.</li>
					<li>5. Copy the <strong>Application (client) ID</strong> and paste it above.</li>
					<li>6. Go to <strong>Authentication</strong>, enable <strong>Allow public client flows</strong>, save.</li>
				</ol>
			</details>
		</div>
	</section>

	<!-- New instance defaults -->
	<section class="flex flex-col gap-3">
		<div class="flex items-center gap-2">
			<h2 class="text-sm font-medium" style="color: var(--text-secondary)">New Instance Defaults</h2>
			<span class="text-xs px-2 py-0.5 rounded" style="background: var(--bg-raised); color: var(--text-muted)">applies to future instances</span>
		</div>

		<!-- RAM -->
		<div class="p-4 rounded-lg border flex flex-col gap-3" style="background: var(--bg-surface); border-color: var(--border)">
			<div class="flex items-center justify-between">
				<span class="text-sm font-medium">Allocated RAM</span>
				<div class="flex items-center gap-1.5">
					<input
						type="number"
						bind:value={ramMb}
						min="512"
						max="65536"
						step="256"
						class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
						style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--accent); outline: none"
					/>
					<span class="text-xs" style="color: var(--text-muted)">MB</span>
				</div>
			</div>
			<input
				type="range"
				bind:value={ramMb}
				min="512"
				max="16384"
				step="256"
				class="w-full accent-[--accent]"
				style="accent-color: var(--accent)"
			/>
			<div class="flex justify-between text-xs" style="color: var(--text-muted)">
				<span>512 MB</span>
				<span>16 GB</span>
			</div>
		</div>

		<!-- Resolution -->
		<div class="p-4 rounded-lg border flex items-center justify-between gap-4"
			style="background: var(--bg-surface); border-color: var(--border)">
			<span class="text-sm font-medium">Resolution</span>
			<div class="flex items-center gap-2">
				<input
					type="number"
					bind:value={resWidth}
					min="640"
					max="7680"
					class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
				/>
				<span class="text-xs" style="color: var(--text-muted)">×</span>
				<input
					type="number"
					bind:value={resHeight}
					min="480"
					max="4320"
					class="w-20 px-2 py-1 rounded text-sm text-right font-mono"
					style="background: var(--bg-raised); border: 1px solid var(--border); color: var(--text-primary); outline: none"
				/>
				<span class="text-xs" style="color: var(--text-muted)">px</span>
			</div>
		</div>

		<!-- Save -->
		<div class="flex items-center justify-end gap-3">
			{#if defaultsError}
				<p class="text-xs flex-1 select-text cursor-text" style="color: var(--danger)">{defaultsError}</p>
			{/if}
			{#if defaultsSaved}
				<p class="text-xs" style="color: var(--success)">Saved. New instances will use these defaults.</p>
			{/if}
			<button
				class="px-4 py-2 rounded text-sm font-medium disabled:opacity-50"
				style="background: var(--accent); color: #fff"
				onclick={saveDefaults}
				disabled={defaultsSaving}
			>
				{defaultsSaving ? 'Saving…' : 'Save defaults'}
			</button>
		</div>
	</section>

	<!-- About -->
	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium" style="color: var(--text-secondary)">About</h2>
		<div class="p-4 rounded-lg border" style="background: var(--bg-surface); border-color: var(--border)">
			<p class="text-sm font-semibold">Cirrus</p>
			<p class="text-xs mt-1" style="color: var(--text-muted)">Version 0.1.0 · Open-source Minecraft launcher</p>
		</div>
	</section>
</div>
