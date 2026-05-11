<script lang="ts">
	import { page } from '$app/stores';

	const links = [
		{
			href: '/',
			label: 'Home',
			icon: `<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/>`,
		},
		{
			href: '/instances',
			label: 'Instances',
			icon: `<rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/>`,
		},
		{
			href: '/mods',
			label: 'Mods',
			icon: `<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/>`,
		},
		{
			href: '/settings',
			label: 'Settings',
			icon: `<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>`,
		},
	];

	const isActive = (href: string) =>
		href === '/'
			? $page.url.pathname === '/'
			: $page.url.pathname.startsWith(href);
</script>

<nav class="sidebar flex flex-col h-full flex-shrink-0 border-r" style="width: 60px; border-color: var(--border)">
	<!-- Logo -->
	<div class="logo-wrap flex items-center justify-center h-14">
		<div class="logo-icon flex items-center justify-center w-9 h-9 rounded-xl">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/>
			</svg>
		</div>
	</div>

	<!-- Separator -->
	<div class="mx-3 h-px" style="background: var(--border)"></div>

	<!-- Nav links -->
	<div class="flex flex-col gap-1 p-2 flex-1 pt-3">
		{#each links as link}
			{@const active = isActive(link.href)}
			<div class="relative group">
				<a
					href={link.href}
					class="nav-link flex items-center justify-center w-full h-10 rounded-lg transition-all"
					class:active
					aria-label={link.label}
				>
					<svg
						width="18" height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.75"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						{@html link.icon}
					</svg>
				</a>

				<!-- Tooltip -->
				<div class="tooltip pointer-events-none absolute left-full top-1/2 ml-3 px-2.5 py-1.5 rounded-lg text-xs font-medium whitespace-nowrap"
					style="
						transform: translateY(-50%);
						background: var(--bg-raised);
						color: var(--text-primary);
						border: 1px solid var(--border-strong);
						box-shadow: 0 8px 24px rgba(0,0,0,0.5);
						z-index: 50;
					"
				>
					{link.label}
					<span class="absolute right-full top-1/2" style="transform: translateY(-50%); border: 5px solid transparent; border-right-color: var(--border-strong); margin-right: -1px;"></span>
					<span class="absolute right-full top-1/2" style="transform: translateY(-50%); border: 4px solid transparent; border-right-color: var(--bg-raised);"></span>
				</div>
			</div>
		{/each}
	</div>
</nav>

<style>
	.sidebar {
		background: var(--bg-surface);
	}

	.logo-icon {
		background: linear-gradient(135deg, #6366f1 0%, #a855f7 100%);
		box-shadow: 0 0 20px rgba(99, 102, 241, 0.35);
	}

	.nav-link {
		color: var(--text-muted);
		position: relative;
	}
	.nav-link:hover {
		color: var(--text-secondary);
		background: var(--bg-hover);
	}
	.nav-link.active {
		color: var(--accent);
		background: var(--accent-dim);
	}
	.nav-link.active::before {
		content: '';
		position: absolute;
		left: -8px;
		top: 25%;
		height: 50%;
		width: 3px;
		border-radius: 0 3px 3px 0;
		background: var(--accent);
	}

	.group:hover .tooltip {
		opacity: 1;
		transform: translateY(-50%) translateX(0);
	}
	.tooltip {
		opacity: 0;
		transform: translateY(-50%) translateX(-4px);
		transition: opacity 0.15s ease, transform 0.15s ease;
	}
</style>
