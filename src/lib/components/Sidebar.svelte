<script lang="ts">
	import { page } from '$app/stores';

	const links = [
		{ href: '/', icon: '⬛', label: 'Home' },
		{ href: '/instances', icon: '📦', label: 'Instances' },
		{ href: '/mods', icon: '🧩', label: 'Mods' },
		{ href: '/settings', icon: '⚙️', label: 'Settings' },
	];
</script>

<nav
	class="flex flex-col h-full border-r flex-shrink-0"
	style="width: 56px; background: var(--bg-surface); border-color: var(--border)"
>
	<!-- Logo -->
	<div class="flex items-center justify-center h-14" style="border-bottom: 1px solid var(--border)">
		<span class="text-xl">☁️</span>
	</div>

	<!-- Nav links -->
	<div class="flex flex-col gap-1 p-2 flex-1">
		{#each links as link}
			{@const active = $page.url.pathname === link.href}
			<div class="relative group">
				<a
					href={link.href}
					class="flex items-center justify-center w-10 h-10 rounded-md transition-colors"
					style="background: {active ? 'var(--bg-raised)' : 'transparent'}; color: {active ? 'var(--text-primary)' : 'var(--text-secondary)'}"
					aria-label={link.label}
				>
					<span class="text-base">{link.icon}</span>
				</a>

				<!-- Tooltip -->
				<div
					class="tooltip pointer-events-none absolute left-full top-1/2 ml-3 px-2.5 py-1.5 rounded-md text-xs font-medium whitespace-nowrap opacity-0 group-hover:opacity-100"
					style="
						transform: translateY(-50%);
						background: var(--bg-raised);
						color: var(--text-primary);
						border: 1px solid var(--border);
						box-shadow: 0 4px 12px rgba(0,0,0,0.4);
						transition: opacity 0.12s ease, transform 0.12s ease;
						transform-origin: left center;
						z-index: 50;
					"
				>
					{link.label}
					<!-- Arrow -->
					<span
						class="absolute right-full top-1/2"
						style="
							transform: translateY(-50%);
							border: 5px solid transparent;
							border-right-color: var(--border);
							margin-right: -1px;
						"
					></span>
					<span
						class="absolute right-full top-1/2"
						style="
							transform: translateY(-50%);
							border: 4px solid transparent;
							border-right-color: var(--bg-raised);
						"
					></span>
				</div>
			</div>
		{/each}
	</div>
</nav>

<style>
	.group:hover .tooltip {
		opacity: 1 !important;
	}
</style>
