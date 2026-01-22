<script lang="ts" generics="T">
    import type { Snippet } from 'svelte';

    let {
        items,
        loading = false,
        item,
        emptyState,
        wrapperClass = 'p-4',
        gridClass = 'grid gap-4 [grid-template-columns:repeat(auto-fit,minmax(9rem,1fr))] sm:[grid-template-columns:repeat(auto-fit,minmax(10rem,1fr))] lg:[grid-template-columns:repeat(auto-fit,minmax(12rem,1fr))]',
        itemClass = '',
        onItemClick,
    }: {
        items: T[];
        loading?: boolean;
        item: Snippet<[T]>;
        emptyState?: Snippet;
        wrapperClass?: string;
        gridClass?: string;
        itemClass?: string;
        onItemClick?: (item: T) => void;
    } = $props();
</script>

<div class={wrapperClass}>
    {#if !loading && items.length === 0}
        {#if emptyState}
            {@render emptyState()}
        {/if}
    {:else}
        <div class={gridClass}>
            {#each items as entry}
                <button
                    class="text-left {itemClass}"
                    onclick={() => onItemClick?.(entry)}
                    disabled={!onItemClick}
                >
                    {@render item(entry)}
                </button>
            {/each}
        </div>
    {/if}
</div>
