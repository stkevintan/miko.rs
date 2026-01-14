<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { onMount, onDestroy } from 'svelte';
    import MainLayout from '../components/MainLayout.svelte';
    import StatCard from '../components/StatCard.svelte';
    import DashboardCard from '../components/DashboardCard.svelte';
    import ConnectionItem from '../components/ConnectionItem.svelte';
    import { authStore } from '../lib/auth.svelte';
    import { toast } from '../lib/toast.svelte';
    import spotifyIcon from '../icons/spotify.svg';
    import lastfmIcon from '../icons/lastfm.svg';
    import neteaseIcon from '../icons/netease.svg';
    import qqmusicIcon from '../icons/qqmusic.svg';

    import api from '../lib/api';
    import type { DashboardData } from '../lib/types';
    import {
        Music,
        User,
        Cpu,
        Play,
        Disc,
        Tag,
        Folder,
    } from 'lucide-svelte';

    let token = $state(localStorage.getItem('token'));
    let data = $state<DashboardData | null>(null);
    let pollInterval: number | null = null;

    async function fetchData() {
        const [stats, system, folders, nowPlaying] = await Promise.all([
            api.get('/stats').catch((err) => {
                console.error('Failed to fetch stats', err);
                toast.error('Failed to fetch library statistics');
                return { data: { songs: 0, albums: 0, artists: 0, genres: 0 } };
            }),
            api.get('/system').catch((err) => {
                console.error('Failed to fetch system info', err);
                toast.error('Failed to fetch system information');
                return {
                    data: {
                        cpu_usage: 0,
                        memory_usage: 0,
                        memory_total: 0,
                    },
                };
            }),
            api.get('/folders').catch((err) => {
                console.error('Failed to fetch folders', err);
                toast.error('Failed to fetch music folders');
                return { data: [] };
            }),
            api.get('/now-playing').catch((err) => {
                console.error('Failed to fetch now playing', err);
                toast.error('Failed to fetch now playing sessions');
                return { data: [] };
            }),
        ]);
        data = {
            stats: stats.data,
            system: system.data,
            folders: folders.data,
            now_playing: nowPlaying.data,
        };
    }

    onMount(() => {
        if (!token) {
            push('/login');
            return;
        }
        authStore.fetchProfile();
        fetchData();
        pollInterval = setInterval(fetchData, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });

    function formatBytes(bytes: number) {
        if (!bytes || bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }
</script>

<MainLayout>
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
            Dashboard
        </h1>
    </div>

    {#if data}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard
                label="Songs"
                value={data.stats.songs}
                color="orange"
                icon={Music}
            />
            <StatCard
                label="Albums"
                value={data.stats.albums}
                color="blue"
                icon={Disc}
            />
            <StatCard
                label="Artists"
                value={data.stats.artists}
                color="green"
                icon={User}
            />
            <StatCard
                label="Genres"
                value={data.stats.genres}
                color="red"
                icon={Tag}
            />
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
            <!-- Top Row: User Profile (6) + App Resources (6) -->
            <div class="lg:col-span-6">
                {#if authStore.loading && !authStore.user}
                    <div
                        class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700 animate-pulse h-full"
                    >
                        <div
                            class="h-6 w-32 bg-gray-200 dark:bg-gray-700 rounded mb-6"
                        ></div>
                        <div class="flex items-start">
                            <div
                                class="w-16 h-16 rounded-2xl bg-gray-200 dark:bg-gray-700 mr-4 shrink-0"
                            ></div>
                            <div class="flex-1 space-y-3">
                                <div
                                    class="h-5 w-24 bg-gray-200 dark:bg-gray-700 rounded"
                                ></div>
                                <div
                                    class="h-4 w-32 bg-gray-200 dark:bg-gray-700 rounded"
                                ></div>
                            </div>
                        </div>
                    </div>
                {:else if authStore.user}
                    <DashboardCard
                        title="Account & Connections"
                        icon={User}
                        iconClass="text-green-600"
                        class="h-full flex flex-col"
                    >
                        <div class="flex-1 flex flex-row flex-wrap items-center justify-start w-full gap-8">
                            <div class="flex items-center min-w-[280px] overflow-hidden">
                                <div
                                    class="w-20 h-20 rounded-3xl bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white text-3xl font-bold mr-5 shrink-0 shadow-lg"
                                >
                                    {authStore.user.username[0].toUpperCase()}
                                </div>
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2">
                                        <p
                                            class="text-xl font-bold text-gray-900 dark:text-white truncate"
                                        >
                                            {authStore.user.username}
                                        </p>
                                        {#if authStore.user.admin}
                                            <span
                                                class="px-1.5 py-0.5 rounded-md bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 text-[10px] font-bold uppercase tracking-wider"
                                            >
                                                Admin
                                            </span>
                                        {/if}
                                    </div>
                                    <p
                                        class="text-sm text-gray-500 dark:text-gray-400 truncate mt-1"
                                    >
                                        {authStore.user.email ||
                                            'No email provided'}
                                    </p>
                                </div>
                            </div>
                            <div class="flex-1"></div>
                            <div
                                class="flex flex-wrap justify-start gap-x-6 gap-y-3 @[620px]:grid @[620px]:grid-cols-2 @[620px]:max-w-[400px]"
                            >
                                <ConnectionItem
                                    name="Netease"
                                    username="小星星OvO"
                                    iconSrc={neteaseIcon}
                                    statusColor="text-red-600"
                                    connected={true}
                                />

                                <ConnectionItem
                                    name="QQ Music"
                                    iconSrc={qqmusicIcon}
                                    statusColor="text-yellow-600"
                                    connected={false}
                                />

                                <ConnectionItem
                                    name="Spotify"
                                    username="stkevintan"
                                    iconSrc={spotifyIcon}
                                    statusColor="text-green-600"
                                    connected={true}
                                />

                                <ConnectionItem
                                    name="Last.fm"
                                    iconSrc={lastfmIcon}
                                    statusColor="text-gray-400"
                                    connected={false}
                                />
                            </div>
                        </div>
                    </DashboardCard>
                {/if}
            </div>

            <div class="lg:col-span-6">
                <DashboardCard
                    title="App Resources"
                    icon={Cpu}
                    iconClass="text-blue-600"
                    class="h-full"
                >
                    <div class="space-y-6">
                        <div>
                            <div class="flex justify-between mb-2">
                                <span
                                    class="text-sm font-medium text-gray-700 dark:text-gray-300"
                                    >CPU Usage</span
                                >
                                <span
                                    class="text-sm font-bold text-blue-600 dark:text-blue-400"
                                    >{data.system.cpu_usage.toFixed(1)}%</span
                                >
                            </div>
                            <div
                                class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden"
                            >
                                <div
                                    class="bg-blue-600 h-2.5 rounded-full transition-all duration-500"
                                    style="width: {Math.min(
                                        data.system.cpu_usage,
                                        100
                                    )}%"
                                ></div>
                            </div>
                        </div>

                        <div>
                            <div class="flex justify-between mb-2">
                                <span
                                    class="text-sm font-medium text-gray-700 dark:text-gray-300"
                                    >Memory Usage</span
                                >
                                <span
                                    class="text-sm font-bold text-purple-600 dark:text-purple-400"
                                >
                                    {formatBytes(data.system.memory_usage)}
                                </span>
                            </div>
                            <div
                                class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden"
                            >
                                <div
                                    class="bg-purple-600 h-2.5 rounded-full transition-all duration-500"
                                    style="width: {(data.system.memory_usage /
                                        data.system.memory_total) *
                                        100}%"
                                ></div>
                            </div>
                            <p
                                class="text-[10px] text-gray-400 mt-1 text-right"
                            >
                                of {formatBytes(data.system.memory_total)} total
                                system RAM
                            </p>
                        </div>
                    </div>
                </DashboardCard>
            </div>

            <!-- Bottom Row: Now Playing (6) + Music Folders (6) -->
            <div class="lg:col-span-6">
                <DashboardCard
                    title="Now Playing"
                    icon={Play}
                    class="h-full max-h-[600px] flex flex-col"
                >
                    {#snippet headerExtra()}
                        <span
                            class="px-2 py-1 text-xs font-medium bg-orange-100 text-orange-600 rounded-full dark:bg-orange-900/30 dark:text-orange-400"
                        >
                            {data?.now_playing.length} Active
                        </span>
                    {/snippet}

                    <div
                        class="flex-1 overflow-y-auto pr-1 -mr-1 custom-scrollbar"
                    >
                        {#if data.now_playing.length > 0}
                            <div class="space-y-4">
                                {#each data.now_playing as session}
                                    <div
                                        class="flex items-center p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600 hover:border-orange-300 dark:hover:border-orange-500/50 transition-all group"
                                    >
                                        <div
                                            class="w-12 h-12 rounded-lg bg-orange-500 flex items-center justify-center text-white mr-4 shrink-0 shadow-md group-hover:scale-105 transition-transform overflow-hidden relative"
                                        >
                                            <Music size={24} />
                                            {#if session.cover_art}
                                                {#key session.cover_art}
                                                    <img
                                                        src="/api/coverart/{session.cover_art}?token={token}"
                                                        alt={session.song_title}
                                                        class="absolute inset-0 w-full h-full object-cover"
                                                        onload={(e) =>
                                                            ((
                                                                e.currentTarget as HTMLImageElement
                                                            ).style.display =
                                                                'block')}
                                                        onerror={(e) =>
                                                            ((
                                                                e.currentTarget as HTMLImageElement
                                                            ).style.display =
                                                                'none')}
                                                    />
                                                {/key}
                                            {/if}
                                        </div>
                                        <div class="min-w-0 flex-1">
                                            <p
                                                class="text-sm font-bold text-gray-900 dark:text-white truncate"
                                            >
                                                {session.song_title ||
                                                    'Unknown Song'}
                                            </p>
                                            <p
                                                class="text-xs text-gray-500 dark:text-gray-400 truncate"
                                            >
                                                {session.artist_name ||
                                                    'Unknown Artist'}
                                            </p>
                                            <div
                                                class="flex items-center mt-1.5 text-[10px] text-gray-400 dark:text-gray-500"
                                            >
                                                <span
                                                    class="bg-gray-200 dark:bg-gray-700 px-1.5 py-0.5 rounded text-gray-600 dark:text-gray-300 mr-2 uppercase font-medium"
                                                >
                                                    {session.username}
                                                </span>
                                                <span class="truncate"
                                                    >via {session.player_name}</span
                                                >
                                            </div>
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {:else}
                            <div
                                class="text-center py-10 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700/30 rounded-xl border border-dashed border-gray-200 dark:border-gray-600"
                            >
                                No active playback sessions
                            </div>
                        {/if}
                    </div>
                </DashboardCard>
            </div>

            <div class="lg:col-span-6">
                <DashboardCard
                    title="Music Folders"
                    icon={Folder}
                    class="h-full max-h-[600px] flex flex-col"
                >
                    <div
                        class="flex-1 overflow-y-auto pr-1 -mr-1 custom-scrollbar"
                    >
                        <div class="space-y-4">
                            {#each data.folders as folder}
                                <div
                                    class="p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600"
                                >
                                    <div
                                        class="flex justify-between items-start mb-2"
                                    >
                                        <div class="min-w-0 flex-1">
                                            <p
                                                class="text-sm font-bold text-gray-900 dark:text-white truncate"
                                            >
                                                {folder.label}
                                            </p>
                                            <p
                                                class="text-xs text-gray-500 dark:text-gray-400 truncate mt-1"
                                                title={folder.path}
                                            >
                                                {folder.path}
                                            </p>
                                        </div>
                                        <div class="ml-4 text-right">
                                            <span
                                                class="px-2.5 py-1 text-xs font-bold bg-orange-100 text-orange-600 rounded-full dark:bg-orange-900/40 dark:text-orange-400"
                                            >
                                                {folder.song_count.toLocaleString()}
                                                songs
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            {/each}
                        </div>
                    </div>
                </DashboardCard>
            </div>
        </div>
    {:else}
        <div class="flex items-center justify-center h-64">
            <div
                class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"
            ></div>
        </div>
    {/if}
</MainLayout>
