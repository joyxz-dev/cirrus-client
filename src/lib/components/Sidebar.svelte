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
	class="sidebar flex flex-col h-full border-r"
	style="
		width: 56px;
		background: var(--bg-surface);
		border-color: var(--border);
		transition: width 0.2s ease;
	"
	onmouseenter={(e) => { (e.currentTarget as HTMLElement).style.width = '200px'; }}
	onmouseleave={(e) => { (e.currentTarget as HTMLElement).style.width = '56px'; }}
>
	<!-- Logo -->
	<div class="flex items-center h-14 px-3 overflow-hidden" style="border-bottom: 1px solid var(--border)">
		<span class="text-xl flex-shrink-0">☁️</span>
		<span class="ml-3 font-semibold text-sm whitespace-nowrap overflow-hidden opacity-0 transition-opacity duration-200" style="color: var(--text-primary)">Cirrus</span>
	</div>

	<!-- Nav links -->
	<div class="flex flex-col gap-1 p-2 flex-1">
		{#each links as link}
			{@const active = $page.url.pathname === link.href}
			<a
				href={link.href}
				class="flex items-center h-10 px-2.5 rounded-md transition-colors overflow-hidden"
				style="
					background: {active ? 'var(--bg-raised)' : 'transparent'};
					color: {active ? 'var(--text-primary)' : 'var(--text-secondary)'};
				"
			>
				<span class="text-base flex-shrink-0">{link.icon}</span>
				<span class="ml-3 text-sm whitespace-nowrap overflow-hidden">{link.label}</span>
			</a>
		{/each}
	</div>
</nav>

<style>
	.sidebar:hover span:last-child,
	.sidebar:hover a span:last-child {
		opacity: 1 !important;
	}
</style>
