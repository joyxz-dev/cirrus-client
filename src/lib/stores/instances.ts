import { writable } from 'svelte/store';
import type { Instance } from '$lib/tauri';

export const instances = writable<Instance[]>([]);
export const selectedInstance = writable<Instance | null>(null);
export const instancesLoading = writable(false);
