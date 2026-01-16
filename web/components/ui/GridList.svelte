<script lang="ts" generics="T">
    import type { Snippet } from 'svelte';

    let {
        items,
        loading = false,
        item,
        emptyState,
        wrapperClass = 'p-4',
        gridClass = 'flex flex-wrap gap-4',
        itemClass = '',
    }: {
        items: T[];
        loading?: boolean;
        item: Snippet<[T]>;
        emptyState?: Snippet;
        wrapperClass?: string;
        gridClass?: string;
        itemClass?: string;
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
                <div class={itemClass}>
                    {@render item(entry)}
                </div>
            {/each}
        </div>
    {/if}
</div>
