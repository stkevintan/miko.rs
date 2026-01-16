<script lang="ts">
    import { LayoutGrid, List } from 'lucide-svelte';
    import {
        libraryViewMode,
        persistLibraryViewMode,
    } from '../../lib/libraryView';
</script>

<div
    class="inline-flex rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 overflow-hidden"
>
    <button
        type="button"
        class="px-3 py-2 text-sm font-semibold flex items-center gap-2 transition-colors
        {$libraryViewMode === 'grid'
            ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
        aria-pressed={$libraryViewMode === 'grid'}
        onclick={() => persistLibraryViewMode('grid')}
    >
        <LayoutGrid size={16} />
    </button>
    <button
        type="button"
        class="px-3 py-2 text-sm font-semibold flex items-center gap-2 transition-colors border-l border-gray-200 dark:border-gray-700
        {$libraryViewMode === 'table'
            ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-white'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
        aria-pressed={$libraryViewMode === 'table'}
        onclick={() => persistLibraryViewMode('table')}
    >
        <List size={16} />
    </button>
</div>
