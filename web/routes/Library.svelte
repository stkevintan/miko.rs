<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../lib/api';
    import { toast } from '../lib/toast.svelte';
    import type { Song, SubsonicResponse, Stats } from '../lib/types';
    import TitleCell from '../components/library/TitleCell.svelte';
    import DurationCell from '../components/library/DurationCell.svelte';
    import DataTable from '../components/ui/DataTable.svelte';
    import Pagination from '../components/ui/Pagination.svelte';

    import { Music } from 'lucide-svelte';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../lib/librarySearch';

    let songs = $state<Song[]>([]);
    let loading = $state(true);
    let totalSongs = $state(0);
    let pageSize = $state(20);
    let currentPage = $state(0);

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

    async function fetchSongs() {
        loading = true;
        try {
            const query = $librarySearchQuery || '';
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
        fetchSongs();
    }

    function handlePageSizeChange(size: number) {
        pageSize = size;
        currentPage = 0;
        fetchSongs();
    }

    onMount(() => {
        fetchStats();
    });

    $effect(() => {
        $librarySearchTrigger;
        currentPage = 0;
        fetchSongs();
    });
</script>

<div class="flex flex-col h-[calc(100vh-8rem)]">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center mb-6 gap-4">
        <h1 class="text-3xl font-extrabold text-gray-900 dark:text-white">
            Library
        </h1>
    </div>

    <!-- Table Container -->
    <div
        class="flex-1 overflow-hidden bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm flex flex-col relative"
    >
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
                    <TitleCell {song} />
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

        <!-- Footer / Pagination -->
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
</div>

<style lang="postcss">
    @reference "../style.css";
</style>
