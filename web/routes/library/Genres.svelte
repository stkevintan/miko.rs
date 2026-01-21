<script lang="ts">
    import { onMount } from 'svelte';
    import { Tag } from 'lucide-svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type { GenreReference, SubsonicResponse } from '../../lib/types';
    import DataTable from '../../components/ui/DataTable.svelte';
    import GridList from '../../components/ui/GridList.svelte';
    import Pagination from '../../components/ui/Pagination.svelte';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../../lib/librarySearch';
    import { libraryViewMode, setLibraryViewKey } from '../../lib/libraryView';

    let genres = $state<GenreReference[]>([]);
    let loading = $state(true);
    let pageSize = $state(50);
    let currentPage = $state(0);
    let searchQuery = $state('');

    const filteredGenres = $derived.by((): GenreReference[] => {
        if (!searchQuery) return genres;
        const q = searchQuery.toLowerCase();
        return genres.filter((g) => g.value.toLowerCase().includes(q));
    });

    const totalGenres = $derived.by(() => filteredGenres.length);
    const pagedGenres = $derived.by((): GenreReference[] =>
        filteredGenres.slice(
            currentPage * pageSize,
            currentPage * pageSize + pageSize,
        ),
    );

    async function fetchGenres() {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getGenres');
            genres = response.data.genres?.genre ?? [];
        } catch (error) {
            console.error('Failed to fetch genres:', error);
            toast.error('Failed to load genres');
        } finally {
            loading = false;
        }
    }

    function handlePageChange(page: number) {
        currentPage = page;
    }

    function handlePageSizeChange(size: number) {
        pageSize = size;
        currentPage = 0;
    }

    onMount(() => {
        setLibraryViewKey('genres');
        fetchGenres();
    });

    $effect(() => {
        $librarySearchTrigger;
        searchQuery = $librarySearchQuery || '';
        currentPage = 0;
    });
</script>

<div
    class="flex-1 min-h-0 overflow-hidden bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm flex flex-col relative"
>
    {#if $libraryViewMode === 'table'}
        <DataTable
            data={pagedGenres}
            {loading}
            minWidth="560px"
            fixed={true}
            resizable={true}
            striped={true}
        >
            {#snippet header()}
                <th>Genre</th>
                <th style="width: 120px" class="text-right">Albums</th>
                <th style="width: 120px" class="text-right">Songs</th>
            {/snippet}

            {#snippet row(genre)}
                <td class="px-4 py-3">
                    <div class="flex items-center gap-3">
                        <div
                            class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-800 flex items-center justify-center"
                        >
                            <Tag
                                size={18}
                                class="text-gray-500 dark:text-gray-400"
                            />
                        </div>
                        <div
                            class="text-sm font-semibold text-gray-900 dark:text-white truncate"
                        >
                            {genre.value}
                        </div>
                    </div>
                </td>
                <td class="px-4 py-3 text-right">
                    <span class="text-sm text-gray-600 dark:text-gray-300">
                        {genre.albumCount}
                    </span>
                </td>
                <td class="px-4 py-3 text-right">
                    <span class="text-sm text-gray-600 dark:text-gray-300">
                        {genre.songCount}
                    </span>
                </td>
            {/snippet}

            {#snippet emptyState()}
                <Tag class="text-gray-300 mb-4" size={48} />
                <p class="text-gray-500 text-lg font-medium">No genres found</p>
                <p class="text-gray-400 text-sm mt-1">
                    Try a different search query
                </p>
            {/snippet}
        </DataTable>
    {:else}
        <GridList
            items={pagedGenres}
            {loading}
            wrapperClass="p-4 overflow-y-auto h-full"
            itemClass="w-full max-w-xs rounded-xl border border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900 p-3"
        >
            {#snippet emptyState()}
                <div class="flex flex-col items-center justify-center py-12">
                    <Tag class="text-gray-300 mb-4" size={48} />
                    <p class="text-gray-500 text-lg font-medium">
                        No genres found
                    </p>
                    <p class="text-gray-400 text-sm mt-1">
                        Try a different search query
                    </p>
                </div>
            {/snippet}

            {#snippet item(genre)}
                <div
                    class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-800 flex items-center justify-center"
                >
                    <Tag size={18} class="text-gray-500 dark:text-gray-400" />
                </div>
                <div class="mt-3 min-w-0">
                    <div
                        class="text-sm font-semibold text-gray-900 dark:text-white truncate"
                    >
                        {genre.value}
                    </div>
                    <div
                        class="text-xs text-gray-500 dark:text-gray-400 truncate"
                    >
                        {genre.albumCount} albums • {genre.songCount} songs
                    </div>
                </div>
            {/snippet}
        </GridList>
    {/if}

    <Pagination
        {currentPage}
        {pageSize}
        totalItems={totalGenres}
        itemCount={pagedGenres.length}
        {loading}
        isSearching={!!$librarySearchQuery}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        unit="genres"
    />
</div>
