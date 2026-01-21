<script lang="ts">
    import { Music, User, Info, Disc } from 'lucide-svelte';
    import type { AlbumWithSongs, SubsonicResponse } from '../../lib/types';
    import { api } from '../../lib/api';
    import Drawer from '../ui/Drawer.svelte';
    import DrawerHeader from '../ui/DrawerHeader.svelte';
    import DrawerSection from '../ui/DrawerSection.svelte';
    import MetaTile from '../ui/MetaTile.svelte';

    let { isOpen = $bindable(false), albumId }: {
        isOpen: boolean;
        albumId: string | null;
    } = $props();

    let album = $state<AlbumWithSongs | null>(null);
    let loading = $state(false);

    $effect(() => {
        if (isOpen && albumId) {
            fetchAlbumDetails(albumId);
        } else if (!isOpen) {
            album = null;
        }
    });

    async function fetchAlbumDetails(id: string) {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getAlbum', {
                params: { id }
            });
            if (response.data.status === 'ok' && response.data.album) {
                album = response.data.album;
            }
        } catch (error) {
            console.error('Failed to fetch album details:', error);
        } finally {
            loading = false;
        }
    }

    function close() {
        isOpen = false;
    }

    function formatDuration(seconds: number | undefined) {
        if (!seconds) return '--:--';
        const hours = Math.floor(seconds / 3600);
        const mins = Math.floor((seconds % 3600) / 60);
        const secs = Math.floor(seconds % 60);
        
        if (hours > 0) {
            return `${hours}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        }
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    }

    function formatSongDuration(seconds: number | undefined) {
        if (!seconds) return '--:--';
        const min = Math.floor(seconds / 60);
        const sec = Math.floor(seconds % 60);
        return `${min}:${sec.toString().padStart(2, '0')}`;
    }
    const uniqueArtists = $derived(() => {
        if (!album || !album.song) return [];
        const artists = new Set<string>();
        if (album.artist) artists.add(album.artist);
        album.song.forEach(s => {
            if (s.artist) artists.add(s.artist);
        });
        return Array.from(artists);
    });
</script>

<Drawer bind:isOpen width="500px">
    {#if loading}
        <div class="h-full flex items-center justify-center">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
        </div>
    {:else if album}
        {#snippet headerIcon()}
            <Disc size={20} />
        {/snippet}
        <DrawerHeader 
            title={album.name} 
            subtitle={album.artist || 'Unknown Artist'}
            icon={headerIcon}
            onClose={close}
        />

        <div class="flex-1 overflow-y-auto p-6 space-y-8">
            <!-- Basic Meta -->
            {#snippet infoIcon()}
                <Info size={14} />
            {/snippet}
            <DrawerSection title="Album Metadata" icon={infoIcon}>
                <div class="grid grid-cols-2 gap-3">
                    <MetaTile label="Total Songs" value="{album.songCount} tracks" />
                    <MetaTile label="Duration" value={formatDuration(album.duration)} />
                    <MetaTile label="Year" value={album.year || 'Unknown'} />
                    <MetaTile label="Genre" value={album.genre || 'None'} />
                </div>
            </DrawerSection>

            <!-- Songs List -->
            {#snippet musicIcon()}
                <Music size={14} />
            {/snippet}
            <DrawerSection title="Tracks" icon={musicIcon}>
                <div class="space-y-1">
                    {#each album.song as song}
                        <div class="group flex items-center gap-3 p-2 hover:bg-gray-50 dark:hover:bg-gray-700/30 rounded-lg transition-colors">
                            <span class="w-6 text-xs text-gray-400 text-center font-mono">
                                {song.track || '-'}
                            </span>
                            <div class="flex-1 min-w-0">
                                <div class="text-sm font-medium dark:text-gray-200 truncate">{song.title}</div>
                            </div>
                            <span class="text-xs text-gray-500 font-mono">
                                {formatSongDuration(song.duration)}
                            </span>
                        </div>
                    {/each}
                </div>
            </DrawerSection>

            <!-- Artists -->
            {#if uniqueArtists().length > 0}
            {#snippet userIcon()}
                <User size={14} />
            {/snippet}
            <DrawerSection title="Artists Involved" icon={userIcon}>
                <div class="flex flex-wrap gap-2">
                    {#each uniqueArtists() as artist}
                        <span class="px-3 py-1 bg-gray-100 dark:bg-gray-700 rounded-full text-xs font-medium dark:text-gray-300">
                            {artist}
                        </span>
                    {/each}
                </div>
            </DrawerSection>
            {/if}
        </div>
    {/if}
</Drawer>
