<script lang="ts">
    import { Music, User, Info, Disc } from 'lucide-svelte';
    import type { ArtistWithAlbums, Song, SubsonicResponse } from '../../lib/types';
    import { api } from '../../lib/api';
    import Drawer from '../ui/Drawer.svelte';
    import DrawerHeader from '../ui/DrawerHeader.svelte';
    import DrawerSection from '../ui/DrawerSection.svelte';
    import MetaTile from '../ui/MetaTile.svelte';
    import CoverArt from '../CoverArt.svelte';

    let { isOpen = $bindable(false), artistId }: {
        isOpen: boolean;
        artistId: string | null;
    } = $props();

    let artist = $state<ArtistWithAlbums | null>(null);
    let songs = $state<Song[]>([]);
    let loading = $state(false);

    $effect(() => {
        if (isOpen && artistId) {
            fetchArtistDetails(artistId);
        } else if (!isOpen) {
            artist = null;
            songs = [];
        }
    });

    async function fetchArtistDetails(id: string) {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getArtist', {
                params: { id }
            });
            if (response.data.status === 'ok' && response.data.artist) {
                artist = response.data.artist;
                // Once we have the name, fetch some popular songs
                fetchArtistSongs(artist.name);
            }
        } catch (error) {
            console.error('Failed to fetch artist details:', error);
        } finally {
            loading = false;
        }
    }

    async function fetchArtistSongs(name: string) {
        try {
            const response = await api.get<SubsonicResponse>('/search3', {
                params: {
                    query: name,
                    songCount: 20,
                    albumCount: 0,
                    artistCount: 0
                }
            });
            if (response.data.searchResult3?.song) {
                // Filter to ensure we only get songs by this exact artist if possible
                songs = response.data.searchResult3.song.filter(s => 
                    s.artist?.toLowerCase().includes(name.toLowerCase())
                );
            }
        } catch (error) {
            console.error('Failed to fetch artist songs:', error);
        }
    }

    function close() {
        isOpen = false;
    }

    function formatSongDuration(seconds: number | undefined) {
        if (!seconds) return '--:--';
        const min = Math.floor(seconds / 60);
        const sec = Math.floor(seconds % 60);
        return `${min}:${sec.toString().padStart(2, '0')}`;
    }
</script>

<Drawer bind:isOpen width="550px">
    {#if loading}
        <div class="h-full flex items-center justify-center">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
        </div>
    {:else if artist}
        {#snippet headerIcon()}
            <User size={20} />
        {/snippet}
        <DrawerHeader 
            title={artist.name} 
            subtitle="Artist Overview"
            icon={headerIcon}
            onClose={close}
        />

        <div class="flex-1 overflow-y-auto p-6 space-y-8">
            <!-- Stats -->
            {#snippet infoIcon()}
                <Info size={14} />
            {/snippet}
            <DrawerSection title="Statistics" icon={infoIcon}>
                <div class="grid grid-cols-2 gap-3">
                    <MetaTile label="Albums" value={artist.albumCount || artist.album?.length || 0} />
                    <MetaTile label="Rating" value={artist.averageRating?.toFixed(1) || 'No rating'} />
                </div>
            </DrawerSection>

            <!-- Top Songs -->
            {#if songs.length > 0}
                {#snippet musicIcon()}
                    <Music size={14} />
                {/snippet}
                <DrawerSection title="Songs" icon={musicIcon}>
                    <div class="space-y-1">
                        {#each songs as song}
                            <div class="group flex items-center gap-3 p-2 hover:bg-gray-50 dark:hover:bg-gray-700/30 rounded-lg transition-colors cursor-default">
                                <CoverArt
                                    id={song.coverArt}
                                    size={14}
                                    class="w-8 h-8 rounded"
                                    fallbackClass="bg-gray-100 dark:bg-gray-800 text-gray-400"
                                    icon={Music}
                                />
                                <div class="flex-1 min-w-0">
                                    <div class="text-xs font-medium dark:text-gray-200 truncate">{song.title}</div>
                                    <div class="text-[10px] text-gray-500 truncate">{song.album}</div>
                                </div>
                                <span class="text-[10px] text-gray-400 font-mono">
                                    {formatSongDuration(song.duration)}
                                </span>
                            </div>
                        {/each}
                    </div>
                </DrawerSection>
            {/if}

            <!-- Albums -->
            {#if artist.album && artist.album.length > 0}
                {#snippet discIcon()}
                    <Disc size={14} />
                {/snippet}
                <DrawerSection title="Albums" icon={discIcon}>
                    <div class="grid grid-cols-2 gap-4">
                        {#each artist.album as album}
                            <div class="flex items-start gap-3 p-2 hover:bg-gray-50 dark:hover:bg-gray-700/30 rounded-xl transition-all group">
                                <CoverArt
                                    id={album.coverArt}
                                    size={20}
                                    class="w-12 h-12 rounded-lg shadow-sm border border-gray-100 dark:border-gray-700 transition-transform group-hover:scale-110"
                                    fallbackClass="bg-gray-100 dark:bg-gray-800 text-gray-400"
                                    icon={Disc}
                                />
                                <div class="min-w-0 pt-1">
                                    <div class="text-xs font-bold text-gray-900 dark:text-white truncate">
                                        {album.name}
                                    </div>
                                    <div class="text-[10px] text-gray-400 line-clamp-1">
                                        {album.year || '—'} • {album.songCount} tracks
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                </DrawerSection>
            {/if}
        </div>
    {/if}
</Drawer>
