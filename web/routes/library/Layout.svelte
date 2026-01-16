<script lang="ts">
    import type { Snippet } from 'svelte';
    import { route } from '../../router';
    import LibraryViewToggle from '@/components/library/LibraryViewToggle.svelte';
    import AlbumSort from '../../components/library/AlbumSort.svelte';

    let { children } = $props<{
        children: Snippet;
    }>();

    const titles: Record<string, string> = {
        '/library/tracks': 'Tracks',
        '/library/albums': 'Albums',
        '/library/artists': 'Artists',
        '/library/genres': 'Genres',
    };

    let currentTitle = $derived(titles[route.pathname] || 'Library');
</script>

<div class="flex flex-col h-[calc(100vh-8rem)]">
    <div class="flex items-center gap-6 mb-4">
        <h2
            class="mr-auto text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
            {currentTitle}
        </h2>
        

        {#if route.pathname === '/library/albums'}
            <AlbumSort />
        {/if}

        <LibraryViewToggle />
    </div>
    {@render children()}
</div>

<style lang="postcss">
    @reference "../../style.css";
</style>
