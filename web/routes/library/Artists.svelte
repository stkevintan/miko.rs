<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type { ArtistReference, SubsonicResponse, Stats } from '../../lib/types';
    import DataTable from '../../components/ui/DataTable.svelte';
    import GridList from '../../components/ui/GridList.svelte';
    import Pagination from '../../components/ui/Pagination.svelte';
    import { Music } from 'lucide-svelte';
    import TitleCell from '../../components/library/TitleCell.svelte';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../../lib/librarySearch';
    import { libraryViewMode, setLibraryViewKey } from '../../lib/libraryView';
    import LibraryViewToggle from '@/components/library/LibraryViewToggle.svelte';

    let artists = $state<ArtistReference[]>([]);
    let loading = $state(true);
    let totalArtists = $state(0);
    let pageSize = $state(20);
    let currentPage = $state(0);
    let searchQuery = $state('');

    async function fetchStats() {
        try {
            const response = await api.get<Stats>('/stats', {
                params: { fields: 'artists' },
            });
            totalArtists = response.data.artists || 0;
        } catch (error) {
            console.error('Failed to fetch stats:', error);
            toast.error('Failed to load library statistics');
        }
    }

    async function fetchArtists(query: string) {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/search3', {
                params: {
                    query: query,
                    songCount: 0,
                    albumCount: 0,
                    artistCount: pageSize,
                    artistOffset: currentPage * pageSize,
                },
            });

            if (response.data.searchResult3?.artist) {
                artists = response.data.searchResult3.artist;
            } else {
                artists = [];
            }
        } catch (error) {
            console.error('Failed to fetch artists:', error);
            toast.error('Failed to load artists from library');
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
        setLibraryViewKey('artists');
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
        fetchArtists(searchQuery);
    });
</script>

<div class="flex items-center mb-4 gap-6">
    <h2 class="mr-auto text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Artists
    </h2>
    <LibraryViewToggle />
</div>

<div
    class="flex-1 min-h-0 overflow-hidden bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm flex flex-col relative"
>
        {#if $libraryViewMode === 'table'}
            <DataTable
                data={artists}
                {loading}
                minWidth="560px"
                fixed={true}
                resizable={true}
            >
                {#snippet header()}
                    <th>Artist</th>
                    <th style="width: 120px" class="text-right">Albums</th>
                    <th style="width: 120px" class="text-right">Rating</th>
                {/snippet}

                {#snippet row(artist)}
                    <td class="px-4 py-3">
                        <TitleCell
                            title={artist.name}
                            coverArt={artist.coverArt}
                            showPlay={false}
                        />
                    </td>
                    <td class="px-6 py-3 text-right">
                        <span class="text-sm text-gray-600 dark:text-gray-300">
                            {artist.albumCount ?? '—'}
                        </span>
                    </td>
                    <td class="px-6 py-3 text-right">
                        <span class="text-sm text-gray-600 dark:text-gray-300">
                            {artist.averageRating !== undefined &&
                            artist.averageRating !== null
                                ? artist.averageRating.toFixed(1)
                                : '—'}
                        </span>
                    </td>
                {/snippet}

                {#snippet emptyState()}
                    <Music class="text-gray-300 mb-4" size={48} />
                    <p class="text-gray-500 text-lg font-medium">No artists found</p>
                    <p class="text-gray-400 text-sm mt-1">
                        Try a different search query
                    </p>
                {/snippet}
            </DataTable>
        {:else}
            <GridList
                items={artists}
                {loading}
                wrapperClass="p-4 overflow-y-auto h-full"
                itemClass="w-36 sm:w-40 md:w-48 lg:w-56 rounded-xl border border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900 p-3"
            >
                {#snippet emptyState()}
                    <div class="flex flex-col items-center justify-center py-12">
                        <Music class="text-gray-300 mb-4" size={48} />
                        <p class="text-gray-500 text-lg font-medium">No artists found</p>
                        <p class="text-gray-400 text-sm mt-1">
                            Try a different search query
                        </p>
                    </div>
                {/snippet}

                {#snippet item(artist)}
                    <TitleCell
                        title={artist.name}
                        coverArt={artist.coverArt}
                        showPlay={false}
                    />
                    <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                        {artist.albumCount ?? '—'} albums
                        <span class="mx-1">•</span>
                        {artist.averageRating !== undefined &&
                        artist.averageRating !== null
                            ? artist.averageRating.toFixed(1)
                            : '—'}
                    </div>
                {/snippet}
            </GridList>
        {/if}

        <Pagination
            {currentPage}
            {pageSize}
            totalItems={totalArtists}
            itemCount={artists.length}
            {loading}
            isSearching={!!$librarySearchQuery}
            onPageChange={handlePageChange}
            onPageSizeChange={handlePageSizeChange}
            unit="artists"
        />
    </div>
