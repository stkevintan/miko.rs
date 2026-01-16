import { writable } from 'svelte/store';

export const librarySearchQuery = writable('');
export const librarySearchTrigger = writable(0);

export const submitLibrarySearch = () => {
    librarySearchTrigger.update((value) => value + 1);
};
