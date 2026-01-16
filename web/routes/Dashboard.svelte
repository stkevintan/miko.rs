<script lang="ts">
    import { navigate } from '../router';
    import { onMount, onDestroy } from 'svelte';
    import StatCard from '../components/StatCard.svelte';
    import AccountConnections from '../components/AccountConnections.svelte';
    import MusicFolders from '../components/MusicFolders.svelte';
    import NowPlaying from '../components/NowPlaying.svelte';
    import AppResources from '../components/AppResources.svelte';
    import { authStore } from '../lib/auth.svelte';
    import { toast } from '../lib/toast.svelte';

    import { api } from '../lib/api';
    import type { Stats } from '../lib/types';
    import { Music, User, Disc, Tag } from 'lucide-svelte';

    let token = $state(localStorage.getItem('token'));
    let stats = $state<Stats | null>(null);
    let pollInterval: number | null = null;

    async function fetchData() {
        try {
            const res = await api.get('/stats');
            stats = res.data;
        } catch (err) {
            console.error('Failed to fetch stats', err);
            toast.error('Failed to fetch library statistics');
            stats = { songs: 0, albums: 0, artists: 0, genres: 0 };
        }
    }

    onMount(() => {
        if (!token) {
            navigate('/login');
            return;
        }
        authStore.fetchProfile();
        fetchData();
        pollInterval = setInterval(fetchData, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });
</script>

<div class="flex items-center justify-between mb-8">
    <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
        Dashboard
    </h1>
</div>

{#if stats}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <a href="/library/tracks" class="block cursor-pointer">
            <StatCard
                label="Songs"
                value={stats.songs}
                color="orange"
                icon={Music}
            />
        </a>
        <a href="/library/albums" class="block cursor-pointer">
            <StatCard
                label="Albums"
                value={stats.albums}
                color="blue"
                icon={Disc}
            />
        </a>
        <a href="/library/artists" class="block cursor-pointer">
            <StatCard
                label="Artists"
                value={stats.artists}
                color="green"
                icon={User}
            />
        </a>
        <a href="/library/genres" class="block cursor-pointer">
            <StatCard
                label="Genres"
                value={stats.genres}
                color="red"
                icon={Tag}
            />
        </a>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- Top Row: User Profile (6) + App Resources (6) -->
        <div class="lg:col-span-6">
            <AccountConnections />
        </div>

        <div class="lg:col-span-6">
            <AppResources />
        </div>

        <!-- Bottom Row: Now Playing (6) + Music Folders (6) -->
        <div class="lg:col-span-6">
            <NowPlaying />
        </div>

        <div class="lg:col-span-6">
            <MusicFolders />
        </div>
    </div>
{:else}
        <div class="flex items-center justify-center h-64">
            <div
                class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"
            ></div>
        </div>
    {/if}
