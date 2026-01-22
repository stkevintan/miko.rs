<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type {
        ArtistReference,
        SubsonicResponse,
        Stats,
    } from '../../lib/types';
    import DataTable from '../../components/ui/DataTable.svelte';
    import GridList from '../../components/ui/GridList.svelte';
    import Pagination from '../../components/ui/Pagination.svelte';
    import { Music, User, Star } from 'lucide-svelte';
    import TitleCell from '../../components/library/TitleCell.svelte';
    import CoverArt from '../../components/CoverArt.svelte';
    import {
        librarySearchQuery,
        librarySearchTrigger,
    } from '../../lib/librarySearch';
    import { libraryViewMode, setLibraryViewKey } from '../../lib/libraryView';
    import ArtistMetadataDrawer from '../../components/library/ArtistMetadataDrawer.svelte';

    let artists = $state<ArtistReference[]>([]);
    let loading = $state(true);
    let totalArtists = $state(0);
    let pageSize = $state(50);
    let currentPage = $state(0);
    let searchQuery = $state('');

    let selectedArtistId = $state<string | null>(null);
    let isDrawerOpen = $state(false);

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
            striped={true}
            onRowClick={(artist) => {
                selectedArtistId = artist.id;
                isDrawerOpen = true;
            }}
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
                        icon={User}
                    />
                </td>
                <td class="px-4 py-3 text-right">
                    <span class="text-sm text-gray-600 dark:text-gray-300">
                        {artist.albumCount ?? '—'}
                    </span>
                </td>
                <td class="px-4 py-3 text-right">
                    <span class="text-sm text-gray-600 dark:text-gray-300">
                        {artist.averageRating?.toFixed(1) ?? '—'}/5
                    </span>
                </td>
            {/snippet}

            {#snippet emptyState()}
                <User class="text-gray-300 mb-4" size={48} />
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
            gridClass="grid gap-6 [grid-template-columns:repeat(auto-fit,minmax(10rem,1fr))] sm:[grid-template-columns:repeat(auto-fit,minmax(12rem,1fr))]"
            itemClass="group w-full max-w-xs rounded-xl border border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-900 p-4 transition-all duration-300 cursor-pointer hover:border-orange-500/40 hover:shadow-xl hover:shadow-orange-500/5"
            onItemClick={(artist) => {
                selectedArtistId = artist.id;
                isDrawerOpen = true;
            }}
        >
            {#snippet emptyState()}
                <div class="flex flex-col items-center justify-center py-12">
                    <Music class="text-gray-300 mb-4" size={48} />
                    <p class="text-gray-500 text-lg font-medium">
                        No artists found
                    </p>
                    <p class="text-gray-400 text-sm mt-1">
                        Try a different search query
                    </p>
                </div>
            {/snippet}

            {#snippet item(artist)}
                <div class="flex flex-col items-center text-center w-full min-w-0">
                    <div
                        class="w-full aspect-square rounded-full overflow-hidden shadow-md border-2 border-transparent group-hover:border-orange-500/20 transition-all duration-500 group-hover:scale-[1.02]"
                    >
                        <CoverArt
                            id={artist.coverArt}
                            size={128}
                            class="w-full h-full object-cover"
                            fallbackClass="bg-gray-100 dark:bg-gray-800 text-gray-400"
                            icon={User}
                        />
                    </div>
                    <div class="mt-3 w-full min-w-0">
                        <div
                            class="text-sm font-bold text-gray-900 dark:text-white truncate px-1"
                        >
                            {artist.name}
                        </div>
                        <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            {artist.albumCount ?? '—'} albums
                            <span class="mx-1">•</span>
                            {artist.averageRating?.toFixed(1) ?? '—'}
                            <Star size={10} class="inline mb-0.5" />
                        </div>
                    </div>
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

<ArtistMetadataDrawer bind:isOpen={isDrawerOpen} artistId={selectedArtistId} />
