import { writable } from 'svelte/store';
import type { Account } from '$lib/tauri';

export const account = writable<Account | null>(null);
export const authLoading = writable(false);
export const authError = writable<string | null>(null);
