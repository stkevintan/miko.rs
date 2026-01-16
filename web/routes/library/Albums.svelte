<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type {
        AlbumReference,
        SubsonicResponse,
        Stats,
    } from '../../lib/types';
    import DurationCell from '../../components/library/DurationCell.svelte';
    import TitleCell from '../../components/library/TitleCell.svelte';
    import CoverArt from '../../components/CoverArt.svelte';
    import DataTable from '../../components/ui/DataTable.svelte';
    import GridList from '../../components/ui/GridList.svelte';
    import Pagination from '../../components/ui/Pagination.svelte';
    import { Music } from 'lucide-svelte';
    import { libraryViewMode, setLibraryViewKey } from '../../lib/libraryView';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../../lib/librarySearch';
    import { albumSortState } from '../../lib/albumSort.svelte';

    let albums = $state<AlbumReference[]>([]);
    let loading = $state(true);
    let totalAlbums = $state(0);
    let pageSize = $state(50);
    let currentPage = $state(0);
    let searchQuery = $state('');

    async function fetchStats() {
        try {
            const response = await api.get<Stats>('/stats', {
                params: { fields: 'albums' },
            });
            totalAlbums = response.data.albums || 0;
        } catch (error) {
            console.error('Failed to fetch stats:', error);
            toast.error('Failed to load library statistics');
        }
    }

    async function fetchAlbums() {
        loading = true;
        try {
            if (searchQuery) {
                const response = await api.get<SubsonicResponse>('/search3', {
                    params: {
                        query: searchQuery,
                        songCount: 0,
                        artistCount: 0,
                        albumCount: pageSize,
                        albumOffset: currentPage * pageSize,
                    },
                });
                albums = response.data.searchResult3?.album ?? [];
            } else {
                const response = await api.get<SubsonicResponse>(
                    '/getAlbumList2',
                    {
                        params: {
                            type: albumSortState.type,
                            size: pageSize,
                            offset: currentPage * pageSize,
                        },
                    },
                );
                albums = response.data.albumList2?.album ?? [];
            }
        } catch (error) {
            console.error('Failed to fetch albums:', error);
            toast.error('Failed to load albums from library');
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
        setLibraryViewKey('albums');
        fetchStats();
    });

    $effect(() => {
        $librarySearchTrigger;
        searchQuery = $librarySearchQuery || '';
        currentPage = 0;
    });

    $effect(() => {
        searchQuery;
        albumSortState.type;
        currentPage;
        pageSize;
        fetchAlbums();
    });
</script>

<div
    class="flex-1 min-h-0 overflow-hidden bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm flex flex-col relative"
>
    {#if $libraryViewMode === 'table'}
        <DataTable
            data={albums}
            {loading}
            minWidth="800px"
            fixed={true}
            resizable={true}
        >
            {#snippet header()}
                <th>Album</th>
                <th style="width: 220px">Artist</th>
                <th style="width: 96px" class="text-right">Year</th>
                <th style="width: 120px" class="text-right">Tracks</th>
                <th style="width: 120px" class="text-right">Duration</th>
            {/snippet}

            {#snippet row(album)}
                <td class="px-4 py-3">
                    <TitleCell
                        title={album.name}
                        coverArt={album.coverArt}
                        showPlay={false}
                    />
                </td>
                <td class="px-6 py-3">
                    <span
                        class="text-sm text-gray-600 dark:text-gray-300 truncate block"
                    >
                        {album.artist || 'Unknown Artist'}
                    </span>
                </td>
                <td class="px-6 py-3 text-right">
                    <span class="text-sm text-gray-500 dark:text-gray-400">
                        {album.year || '—'}
                    </span>
                </td>
                <td class="px-6 py-3 text-right">
                    <span class="text-sm text-gray-500 dark:text-gray-400">
                        {album.songCount ?? '—'}
                    </span>
                </td>
                <td class="px-6 py-3 text-right">
                    <div class="no-truncate">
                        <DurationCell duration={album.duration} />
                    </div>
                </td>
            {/snippet}

            {#snippet emptyState()}
                <Music class="text-gray-300 mb-4" size={48} />
                <p class="text-gray-500 text-lg font-medium">No albums found</p>
                <p class="text-gray-400 text-sm mt-1">
                    Try a different search query
                </p>
            {/snippet}
        </DataTable>
    {:else}
        <GridList
            items={albums}
            {loading}
            wrapperClass="p-4 overflow-y-auto h-full"
            itemClass="w-full max-w-xs rounded-xl border border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900 p-3"
        >
            {#snippet emptyState()}
                <div class="flex flex-col items-center justify-center py-12">
                    <Music class="text-gray-300 mb-4" size={48} />
                    <p class="text-gray-500 text-lg font-medium">
                        No albums found
                    </p>
                    <p class="text-gray-400 text-sm mt-1">
                        Try a different search query
                    </p>
                </div>
            {/snippet}

            {#snippet item(album)}
                <CoverArt
                    id={album.coverArt}
                    alt={album.name}
                    size={24}
                    class="w-full aspect-square rounded-lg"
                />
                <div class="mt-3 min-w-0">
                    <div
                        class="text-sm font-semibold text-gray-900 dark:text-white truncate"
                    >
                        {album.name}
                    </div>
                    <div
                        class="text-xs text-gray-500 dark:text-gray-400 truncate"
                    >
                        {album.artist || 'Unknown Artist'}
                    </div>
                    <div
                        class="text-xs text-gray-400 dark:text-gray-500 truncate"
                    >
                        {album.year || '—'} • {album.songCount ?? '—'} tracks
                    </div>
                    <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                        <DurationCell duration={album.duration} />
                    </div>
                </div>
            {/snippet}
        </GridList>
    {/if}

    <Pagination
        {currentPage}
        {pageSize}
        totalItems={totalAlbums}
        itemCount={albums.length}
        {loading}
        isSearching={!!$librarySearchQuery}
        onPageChange={handlePageChange}
        onPageSizeChange={handlePageSizeChange}
        unit="albums"
    />
</div>
