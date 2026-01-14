<script lang="ts">
    import { ChevronLeft, ChevronRight, Loader2 } from 'lucide-svelte';

    let {
        currentPage,
        pageSize,
        totalItems,
        loading = false,
        itemCount = 0,
        isSearching = false,
        onPageChange,
        onPageSizeChange,
        unit = 'items'
    }: {
        currentPage: number;
        pageSize: number;
        totalItems: number;
        loading?: boolean;
        itemCount: number;
        isSearching?: boolean;
        onPageChange: (page: number) => void;
        onPageSizeChange: (size: number) => void;
        unit?: string;
    } = $props();

    const totalPages = $derived(Math.ceil(totalItems / pageSize));
    const start = $derived(currentPage * pageSize + 1);
    const end = $derived(currentPage * pageSize + itemCount);
    const hasMore = $derived(isSearching ? itemCount === pageSize : end < totalItems);

    function handlePageSizeChange(e: Event) {
        const newSize = parseInt((e.target as HTMLSelectElement).value);
        onPageSizeChange(newSize);
    }

    function handleJump(e: Event) {
        e.preventDefault();
        const input = (e.target as HTMLFormElement).querySelector('input') as HTMLInputElement;
        let page = parseInt(input.value) - 1;
        if (isNaN(page)) return;
        
        // Clamp page
        page = Math.max(0, Math.min(page, totalPages - 1));
        onPageChange(page);
        input.value = (page + 1).toString();
    }
</script>

<div class="px-6 py-4 border-t border-gray-100 dark:border-gray-800 flex flex-col sm:flex-row items-center justify-between gap-4 bg-gray-50/50 dark:bg-gray-900/50 backdrop-blur-sm">
    <div class="flex items-center gap-4 flex-wrap justify-center sm:justify-start">
        <div class="text-sm text-gray-500 dark:text-gray-400 flex items-center gap-2">
            {#if loading && itemCount > 0}
                <Loader2 class="animate-spin text-orange-500" size={14} />
                <span>Updating...</span>
            {:else if isSearching}
                <span>Found <span class="font-medium text-gray-900 dark:text-white">{itemCount}</span> results</span>
            {:else if itemCount > 0}
                <span>Showing <span class="font-medium text-gray-900 dark:text-white">{start} - {end}</span> of <span class="font-medium text-gray-900 dark:text-white">{totalItems}</span> {unit}</span>
            {/if}
        </div>

        <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 border-l border-gray-200 dark:border-gray-700 pl-4">
            <label for="pageSize">Show</label>
            <select
                id="pageSize"
                value={pageSize}
                onchange={handlePageSizeChange}
                class="bg-transparent border-none focus:ring-0 cursor-pointer font-medium text-gray-900 dark:text-white py-0 pl-0 pr-6"
            >
                {#each [20, 50, 100, 500] as size}
                    <option value={size} class="bg-white dark:bg-gray-900">{size}</option>
                {/each}
            </select>
        </div>
    </div>
    
    <div class="flex items-center gap-6">
        <form onsubmit={handleJump} class="hidden md:flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
            <label for="jumpTo">Go to</label>
            <input
                id="jumpTo"
                type="number"
                min="1"
                max={totalPages}
                value={currentPage + 1}
                class="w-12 h-8 text-center bg-gray-100 dark:bg-gray-800 border-none rounded-lg focus:ring-2 focus:ring-orange-500 outline-none transition-all text-gray-900 dark:text-white p-0 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
            />
            <span>of {totalPages || 1}</span>
        </form>

        <div class="flex items-center gap-4">
            <span class="text-sm text-gray-500 dark:text-gray-400 sm:hidden">
                Page <span class="font-medium text-gray-900 dark:text-white">{currentPage + 1}</span> / {totalPages || 1}
            </span>
            <div class="flex items-center gap-1">
                <button
                    onclick={() => onPageChange(currentPage - 1)}
                    disabled={currentPage === 0 || loading}
                    class="p-1.5 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-white dark:hover:bg-gray-800 disabled:opacity-30 disabled:cursor-not-allowed transition-all shadow-sm"
                    aria-label="Previous page"
                >
                    <ChevronLeft size={18} class="text-gray-600 dark:text-gray-300" />
                </button>
                <button
                    onclick={() => onPageChange(currentPage + 1)}
                    disabled={loading || !hasMore}
                    class="p-1.5 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-white dark:hover:bg-gray-800 disabled:opacity-30 disabled:cursor-not-allowed transition-all shadow-sm"
                    aria-label="Next page"
                >
                    <ChevronRight size={18} class="text-gray-600 dark:text-gray-300" />
                </button>
            </div>
        </div>
    </div>
</div>

<style lang="postcss">
    @reference "../../style.css";
</style>
