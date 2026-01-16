<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type { Song, SubsonicResponse, Stats } from '../../lib/types';
    import TitleCell from '../../components/library/TitleCell.svelte';
    import DurationCell from '../../components/library/DurationCell.svelte';
    import CoverArt from '../../components/CoverArt.svelte';
    import DataTable from '../../components/ui/DataTable.svelte';
    import GridList from '../../components/ui/GridList.svelte';
    import Pagination from '../../components/ui/Pagination.svelte';
    import { Music } from 'lucide-svelte';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../../lib/librarySearch';
    import { libraryViewMode, setLibraryViewKey } from '../../lib/libraryView';
    import LibraryViewToggle from '@/components/library/LibraryViewToggle.svelte';

    let songs = $state<Song[]>([]);
    let loading = $state(true);
    let totalSongs = $state(0);
    let pageSize = $state(20);
    let currentPage = $state(0);
    let searchQuery = $state('');

    async function fetchStats() {
        try {
            const response = await api.get<Stats>('/stats', {
                params: { fields: 'songs' },
            });
            totalSongs = response.data.songs || 0;
        } catch (error) {
            console.error('Failed to fetch stats:', error);
            toast.error('Failed to load library statistics');
        }
    }

    async function fetchSongs(query: string) {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/search3', {
                params: {
                    query: query,
                    albumCount: 0,
                    artistCount: 0,
                    songCount: pageSize,
                    songOffset: currentPage * pageSize,
                },
            });

            if (response.data.searchResult3?.song) {
                songs = response.data.searchResult3.song;
            } else {
                songs = [];
            }
        } catch (error) {
            console.error('Failed to fetch songs:', error);
            toast.error('Failed to load songs from library');
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
        setLibraryViewKey('tracks');
        fetchStats();
    });

    $effect(() => {
        $librarySearchTrigger;
        searchQuery = $librarySearchQuery || '';
        currentPage = 0;
    });

    $effect(() => {
        searchQuery;
        currentPage;
        pageSize;
        fetchSongs(searchQuery);
    });
</script>

<div class="flex items-center gap-6 mb-4">
      <h2 class="mr-auto text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Tracks
    </h2>
    <LibraryViewToggle />
</div>
<div
    class="flex-1 min-h-0 overflow-hidden bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm flex flex-col relative"
>
        {#if $libraryViewMode === 'table'}
            <DataTable
                data={songs}
                {loading}
                minWidth="800px"
                fixed={true}
                resizable={true}
            >
                {#snippet header()}
                    <th>Title</th>
                    <th style="width: 192px">Artist</th>
                    <th style="width: 256px">Album</th>
                    <th style="width: 120px" class="text-right">Duration</th>
                {/snippet}

                {#snippet row(song)}
                    <td class="px-4 py-3">
                        <TitleCell title={song.title} coverArt={song.coverArt} />
                    </td>
                    <td class="px-6 py-3">
                        <span
                            class="text-sm text-gray-600 dark:text-gray-300 truncate block"
                            >{song.artist}</span
                        >
                    </td>
                    <td class="px-6 py-3">
                        <span
                            class="text-sm text-gray-500 dark:text-gray-400 truncate block"
                            >{song.album}</span
                        >
                    </td>
                    <td class="px-6 py-3 text-right">
                        <div class="no-truncate">
                            <DurationCell duration={song.duration} />
                        </div>
                    </td>
                {/snippet}

                {#snippet emptyState()}
                    <Music class="text-gray-300 mb-4" size={48} />
                    <p class="text-gray-500 text-lg font-medium">No songs found</p>
                    <p class="text-gray-400 text-sm mt-1">
                        Try a different search query
                    </p>
                {/snippet}
            </DataTable>
        {:else}
            <GridList
                items={songs}
                {loading}
                wrapperClass="p-4 overflow-y-auto h-full"
                itemClass="w-full max-w-xs rounded-xl border border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900 p-3"
            >
                {#snippet emptyState()}
                    <div class="flex flex-col items-center justify-center py-12">
                        <Music class="text-gray-300 mb-4" size={48} />
                        <p class="text-gray-500 text-lg font-medium">No songs found</p>
                        <p class="text-gray-400 text-sm mt-1">
                            Try a different search query
                        </p>
                    </div>
                {/snippet}

                {#snippet item(song)}
                    <CoverArt
                        id={song.coverArt}
                        alt={song.title}
                        size={24}
                        class="w-full aspect-square rounded-lg"
                    />
                    <div class="mt-3 min-w-0">
                        <div class="text-sm font-semibold text-gray-900 dark:text-white truncate">
                            {song.title}
                        </div>
                        <div class="text-xs text-gray-500 dark:text-gray-400 truncate">
                            {song.artist}
                        </div>
                        <div class="text-xs text-gray-400 dark:text-gray-500 truncate">
                            {song.album}
                        </div>
                        <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            <DurationCell duration={song.duration} />
                        </div>
                    </div>
                {/snippet}
            </GridList>
        {/if}

        <Pagination
            {currentPage}
            {pageSize}
            totalItems={totalSongs}
            itemCount={songs.length}
            {loading}
            isSearching={!!$librarySearchQuery}
            onPageChange={handlePageChange}
            onPageSizeChange={handlePageSizeChange}
            unit="songs"
        />
    </div>
