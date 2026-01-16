import { writable } from 'svelte/store';

export type LibraryViewMode = 'table' | 'grid';

const DEFAULT_MODE: LibraryViewMode = 'grid';
let currentKey = 'tracks';

export const libraryViewMode = writable<LibraryViewMode>(DEFAULT_MODE);

const isValidMode = (value: string | null): value is LibraryViewMode =>
	value === 'table' || value === 'grid';

function readMode(key: string): LibraryViewMode {
	if (typeof localStorage === 'undefined') return DEFAULT_MODE;
	const value = localStorage.getItem(`libraryViewMode:${key}`);
	return isValidMode(value) ? value : DEFAULT_MODE;
}

export function setLibraryViewKey(key: string) {
	currentKey = key;
	libraryViewMode.set(readMode(key));
}

export function persistLibraryViewMode(mode: LibraryViewMode) {
	if (typeof localStorage !== 'undefined') {
		localStorage.setItem(`libraryViewMode:${currentKey}`, mode);
	}
	libraryViewMode.set(mode);
}
