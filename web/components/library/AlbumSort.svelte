<script lang="ts">
    import { albumSortState } from '../../lib/albumSort.svelte';

    const sortOptions = [
        { label: 'Newest', value: 'newest' },
        { label: 'Recently Played', value: 'recent' },
        { label: 'Most Played', value: 'frequent' },
        { label: 'Random', value: 'random' },
        { label: 'Starred', value: 'starred' },
        { label: 'A–Z (Album)', value: 'alphabeticalByName' },
        { label: 'A–Z (Artist)', value: 'alphabeticalByArtist' },
        { label: 'By Year', value: 'byYear' },
    ] as const;

    function handleSortChange(event: Event) {
        albumSortState.type = (event.target as HTMLSelectElement).value as any;
    }
</script>

<div class="flex items-center">
    <label
        class="text-xs font-medium text-gray-500 dark:text-gray-400 mr-2 uppercase tracking-wider"
        for="albumSort"
    >
        Sort
    </label>
    <select
        id="albumSort"
        value={albumSortState.type}
        onchange={handleSortChange}
        class="rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm text-gray-900 dark:text-white px-3 py-1.5 focus:ring-2 focus:ring-orange-500 outline-none transition-shadow"
    >
        {#each sortOptions as option}
            <option value={option.value}>{option.label}</option>
        {/each}
    </select>
</div>
