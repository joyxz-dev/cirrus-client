<script lang="ts">
	import { onMount } from 'svelte';
	import { instances, selectedInstance } from '$lib/stores/instances';
	import { commands } from '$lib/tauri';

	const selected = $derived($selectedInstance ?? $instances[0] ?? null);

	// Azure client ID
	let clientId = $state('');
	let clientIdSaved = $state(false);
	let clientIdSaving = $state(false);
	let clientIdError = $state('');
	let showClientIdInput = $state(false);

	// Instance settings (local edits before saving)
	let editingId = $state<string | null>(null);
	let ramMb = $state(2048);
	let resWidth = $state(1920);
	let resHeight = $state(1080);
	let settingsSaving = $state(false);
	let settingsSaved = $state(false);
	let settingsError = $state('');

	onMount(async () => {
		try {
			const stored = await commands.getClientId();
			if (stored) clientId = stored;
		} catch {}
	});

	// Sync local state when selected instance changes
	$effect(() => {
		if (selected && selected.id !== editingId) {
			editingId = selected.id;
			ramMb = selected.allocatedRamMb;
			resWidth = selected.resolution.width;
			resHeight = selected.resolution.height;
			settingsSaved = false;
			settingsError = '';
		}
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

	async function saveInstanceSettings() {
		if (!selected) return;
		settingsSaving = true;
		settingsSaved = false;
		settingsError = '';
		try {
			const updated = await commands.updateInstance(selected.id, {
				allocatedRamMb: ramMb,
				resolutionWidth: resWidth,
				resolutionHeight: resHeight,
			});
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
			settingsSaved = true;
			setTimeout(() => (settingsSaved = false), 3000);
		} catch (e) {
			settingsError = String(e);
		} finally {
			settingsSaving = false;
		}
	}

	async function toggleSync() {
		if (!selected) return;
		const newVal = !selected.syncOptions;
		if (newVal) {
			const ok = confirm(
				"Enabling sync will replace this instance's options.txt with the shared global settings. Continue?"
			);
			if (!ok) return;
		}
		try {
			await commands.setSyncOptions(selected.id, newVal);
			const updated = await commands.getInstance(selected.id);
			instances.update((list) => list.map((i) => (i.id === updated.id ? updated : i)));
			selectedInstance.set(updated);
		} catch (e) {
			alert(String(e));
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

	<!-- Instance settings -->
	{#if selected}
		<section class="flex flex-col gap-3">
			<h2 class="text-sm font-medium" style="color: var(--text-secondary)">
				Instance — {selected.name}
			</h2>

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

			<!-- Sync -->
			<div class="flex items-center justify-between p-4 rounded-lg border"
				style="background: var(--bg-surface); border-color: var(--border)">
				<div class="flex flex-col gap-0.5">
					<span class="text-sm font-medium">Sync options.txt</span>
					<span class="text-xs" style="color: var(--text-secondary)">
						Share game settings across all instances with sync enabled.
					</span>
				</div>
				<button
					class="relative w-10 h-6 rounded-full transition-colors flex-shrink-0"
					style="background: {selected.syncOptions ? 'var(--accent)' : 'var(--bg-raised)'}"
					onclick={toggleSync}
					aria-label="Toggle sync"
				>
					<span
						class="absolute top-1 w-4 h-4 rounded-full bg-white transition-transform"
						style="transform: translateX({selected.syncOptions ? '18px' : '4px'})"
					></span>
				</button>
			</div>

			<!-- Save -->
			<div class="flex items-center justify-end gap-3">
				{#if settingsError}
					<p class="text-xs flex-1 select-text cursor-text" style="color: var(--danger)">{settingsError}</p>
				{/if}
				{#if settingsSaved}
					<p class="text-xs" style="color: var(--success)">Saved.</p>
				{/if}
				<button
					class="px-4 py-2 rounded text-sm font-medium disabled:opacity-50"
					style="background: var(--accent); color: #fff"
					onclick={saveInstanceSettings}
					disabled={settingsSaving}
				>
					{settingsSaving ? 'Saving…' : 'Save changes'}
				</button>
			</div>
		</section>
	{:else}
		<p class="text-sm" style="color: var(--text-muted)">Select an instance on the Home page to configure it.</p>
	{/if}

	<!-- About -->
	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-medium" style="color: var(--text-secondary)">About</h2>
		<div class="p-4 rounded-lg border" style="background: var(--bg-surface); border-color: var(--border)">
			<p class="text-sm font-semibold">Cirrus</p>
			<p class="text-xs mt-1" style="color: var(--text-muted)">Version 0.1.0 · Open-source Minecraft launcher</p>
		</div>
	</section>
</div>
