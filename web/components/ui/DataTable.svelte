<script lang="ts" generics="T">
    import { type Snippet } from 'svelte';
    import { Loader2 } from 'lucide-svelte';

    let {
        data,
        loading = false,
        header,
        row,
        emptyState,
    }: {
        data: T[];
        loading?: boolean;
        header: Snippet;
        row: Snippet<[T]>;
        emptyState?: Snippet;
    } = $props();
</script>

<div class="flex-1 overflow-auto data-table-container custom-scrollbar relative">
    <table class="w-full text-left border-separate border-spacing-0">
        <thead class="sticky top-0 z-10 bg-white/80 dark:bg-gray-900/80 backdrop-blur-md">
            <tr>
                {@render header()}
            </tr>
        </thead>
        <tbody class="divide-y divide-gray-50 dark:divide-gray-800">
            {#each data as item}
                <tr class="group hover:bg-orange-50/30 dark:hover:bg-orange-500/5 transition-colors cursor-default">
                    {@render row(item)}
                </tr>
            {/each}
        </tbody>
    </table>

    {#if loading}
        <div class="absolute inset-0 flex items-center justify-center bg-white/50 dark:bg-gray-900/50 z-20">
            <Loader2 class="animate-spin text-orange-500" size={32} />
        </div>
    {:else if data.length === 0}
        {#if emptyState}
            <div class="flex flex-col items-center justify-center py-20">
                {@render emptyState()}
            </div>
        {/if}
    {/if}
</div>

<style lang="postcss">
    @reference "../../style.css";

    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }

    .custom-scrollbar::-webkit-scrollbar-track {
        @apply bg-transparent;
    }

    .custom-scrollbar::-webkit-scrollbar-thumb {
        @apply bg-gray-200 dark:bg-gray-800 rounded-full;
    }

    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        @apply bg-gray-300 dark:bg-gray-700;
    }

    .data-table-container :global(th) {
        @apply px-6 py-4 text-xs font-semibold uppercase tracking-wider text-gray-500 border-b border-gray-100 dark:border-gray-800;
    }

    .data-table-container :global(th:first-child) {
        @apply pl-6;
    }
</style>
