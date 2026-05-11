import { invoke } from '@tauri-apps/api/core';

export interface Account {
	uuid: string;
	username: string;
	expiresAt: number;
}

export interface Resolution {
	width: number;
	height: number;
}

export type Loader = 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge';

export interface InstalledMod {
	id: string;
	name: string;
	version: string;
	filename: string;
	sha512: string;
}

export interface Instance {
	id: string;
	name: string;
	mcVersion: string;
	loader: Loader;
	loaderVersion?: string;
	mods: InstalledMod[];
	jvmArgs: string[];
	allocatedRamMb: number;
	syncOptions: boolean;
	resolution: Resolution;
	createdAt: number;
	lastPlayedAt?: number;
	iconPath?: string;
}

export interface DeviceCodeChallenge {
	user_code: string;
	verification_uri: string;
	message: string;
}

export interface ModSearchQuery {
	query: string;
	loader?: string;
	mcVersion?: string;
	category?: string;
	index?: string;
	offset: number;
	limit: number;
}

export interface ModResult {
	id: string;
	slug: string;
	title: string;
	description: string;
	iconUrl?: string;
	downloads: number;
	categories: string[];
}

export interface ModSearchResult {
	hits: ModResult[];
	totalHits: number;
	offset: number;
	limit: number;
}

export interface ModVersionFile {
	url: string;
	filename: string;
	hashes: { sha512: string; sha1?: string };
	primary: boolean;
}

export interface ModVersion {
	id: string;
	name: string;
	versionNumber: string;
	files: ModVersionFile[];
}

export interface DownloadProgress {
	phase: string;
	current: number;
	total: number;
}

export const commands = {
	getAccount: () => invoke<Account | null>('get_account'),
	getClientId: () => invoke<string | null>('get_client_id'),
	setClientId: (clientId: string) => invoke<void>('set_client_id', { clientId }),
	startAuth: () => invoke<DeviceCodeChallenge>('start_auth'),
	logout: () => invoke<void>('logout'),

	listInstances: () => invoke<Instance[]>('list_instances'),
	createInstance: (name: string, mcVersion: string, loader: Loader, loaderVersion?: string) =>
		invoke<Instance>('create_instance', { name, mcVersion, loader, loaderVersion }),
	updateInstance: (id: string, opts: { name?: string; allocatedRamMb?: number; resolutionWidth?: number; resolutionHeight?: number }) =>
		invoke<Instance>('update_instance', { id, ...opts }),
	deleteInstance: (id: string) => invoke<void>('delete_instance', { id }),
	getInstance: (id: string) => invoke<Instance>('get_instance', { id }),
	launchInstance: (id: string) => invoke<void>('launch_instance', { id }),
	downloadVersion: (mcVersion: string) => invoke<void>('download_version', { mcVersion }),
	getVersionList: () => invoke<{ id: string; kind: string }[]>('get_version_list'),
	setSyncOptions: (instanceId: string, enabled: boolean) =>
		invoke<void>('set_sync_options', { instanceId, enabled }),

	searchMods: (query: ModSearchQuery) => invoke<ModSearchResult>('search_mods', { query }),
	getModVersions: (projectId: string, loader?: string, mcVersion?: string) =>
		invoke<ModVersion[]>('get_mod_versions', { projectId, loader, mcVersion }),
	installMod: (instanceId: string, projectId: string, versionId?: string) =>
		invoke<InstalledMod>('install_mod', { instanceId, projectId, versionId }),
	removeMod: (instanceId: string, modId: string) =>
		invoke<void>('remove_mod', { instanceId, modId }),
};
