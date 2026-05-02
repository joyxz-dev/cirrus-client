import { writable } from 'svelte/store';

export type LaunchState = 'idle' | 'downloading' | 'launching' | 'running' | 'error';

export const launchState = writable<LaunchState>('idle');
export const launchProgress = writable(0);
export const launchMessage = writable('');
export const launchError = writable<string | null>(null);
