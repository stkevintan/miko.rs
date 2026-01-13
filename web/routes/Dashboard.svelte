<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { onMount, onDestroy } from 'svelte';
    import MainLayout from '../components/MainLayout.svelte';
    import StatCard from '../components/StatCard.svelte';
    import { authStore } from '../lib/auth.svelte';
    import api from '../lib/api';
    import type { DashboardData } from '../lib/types';
    import { Music, Library, User, Cpu, Zap, Play, Disc, List } from 'lucide-svelte';

    let token = $state(localStorage.getItem('token'));
    let data = $state<DashboardData | null>(null);
    let pollInterval: any;

    async function fetchData() {
        try {
            const [stats, system, nowPlaying] = await Promise.all([
                api.get('/stats'),
                api.get('/system'),
                api.get('/now-playing')
            ]);
            data = {
                stats: stats.data,
                system: system.data,
                now_playing: nowPlaying.data
            };
        } catch (e) {
            console.error('Failed to fetch dashboard data', e);
        }
    }

    onMount(() => {
        if (!token) {
            push('/login');
            return;
        }
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
    <div class="mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
    </div>

    {#if data}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <StatCard label="Songs" value={data.stats.songs} color="orange" icon={Music} />
            <StatCard label="Albums" value={data.stats.albums} color="blue" icon={Disc} />
            <StatCard label="Artists" value={data.stats.artists} color="green" icon={User} />
            <StatCard label="Playlists" value={data.stats.playlists} color="purple" icon={List} />
        </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {#if authStore.user}
            <!-- User Profile section -->
            <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center mb-6">
                    <User size={20} class="mr-2 text-green-600" />
                    Current User
                </h2>
                <div class="flex items-start">
                    <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white text-2xl font-bold mr-4 shrink-0 shadow-lg">
                        {authStore.user.username[0].toUpperCase()}
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-lg font-bold text-gray-900 dark:text-white truncate">{authStore.user.username}</p>
                        <p class="text-sm text-gray-500 dark:text-gray-400 truncate mb-3">{authStore.user.email || 'No email provided'}</p>
                        
                        <div class="flex flex-wrap gap-2">
                            {#each authStore.user.roles as role}
                                <span class="px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider bg-gray-100 text-gray-600 rounded-full dark:bg-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-600">
                                    {role}
                                </span>
                            {/each}
                        </div>
                    </div>
                </div>
            </div>
        {/if}

        <!-- System Info section -->
        <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700">
            <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center mb-6">
                <Cpu size={20} class="mr-2 text-blue-600" />
                App Resources
            </h2>
            
            <div class="space-y-6">
                <div>
                    <div class="flex justify-between mb-2">
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">CPU Usage</span>
                        <span class="text-sm font-bold text-blue-600 dark:text-blue-400">{data.system.cpu_usage.toFixed(1)}%</span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden">
                        <div class="bg-blue-600 h-2.5 rounded-full transition-all duration-500" style="width: {Math.min(data.system.cpu_usage, 100)}%"></div>
                    </div>
                </div>

                <div>
                    <div class="flex justify-between mb-2">
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Memory Usage</span>
                        <span class="text-sm font-bold text-purple-600 dark:text-purple-400">
                            {formatBytes(data.system.memory_usage)}
                        </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden">
                        <div class="bg-purple-600 h-2.5 rounded-full transition-all duration-500" style="width: {(data.system.memory_usage / data.system.memory_total) * 100}%"></div>
                    </div>
                    <p class="text-[10px] text-gray-400 mt-1 text-right">of {formatBytes(data.system.memory_total)} total system RAM</p>
                </div>
            </div>
        </div>
    </div>

    <!-- Now Playing section -->
    <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700">
        <div class="flex items-center justify-between mb-6">
            <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center">
                <Play size={20} class="mr-2 text-orange-600" />
                Now Playing
            </h2>
            <span class="px-2 py-1 text-xs font-medium bg-orange-100 text-orange-600 rounded-full dark:bg-orange-900/30 dark:text-orange-400">
                {data.now_playing.length} Active
            </span>
        </div>
        
        {#if data.now_playing.length > 0}
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {#each data.now_playing as session}
                    <div class="flex items-center p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600 hover:border-orange-300 dark:hover:border-orange-500/50 transition-all group">
                        <div class="w-12 h-12 rounded-lg bg-orange-500 flex items-center justify-center text-white mr-4 shrink-0 shadow-md group-hover:scale-105 transition-transform overflow-hidden relative">
                            {#if session.cover_art}
                                <img 
                                    src="/api/coverart/{session.cover_art}?token={token}" 
                                    alt={session.song_title}
                                    class="w-full h-full object-cover"
                                    onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'}
                                />
                                <div class="absolute inset-0 flex items-center justify-center -z-10 bg-orange-500">
                                    <Music size={24} />
                                </div>
                            {:else}
                                <Music size={24} />
                            {/if}
                        </div>
                        <div class="min-w-0 flex-1">
                            <p class="text-sm font-bold text-gray-900 dark:text-white truncate">
                                {session.song_title || 'Unknown Song'}
                            </p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                {session.artist_name || 'Unknown Artist'}
                            </p>
                            <div class="flex items-center mt-1.5 text-[10px] text-gray-400 dark:text-gray-500">
                                <span class="bg-gray-200 dark:bg-gray-700 px-1.5 py-0.5 rounded text-gray-600 dark:text-gray-300 mr-2 uppercase font-medium">
                                    {session.username}
                                </span>
                                <span class="truncate">via {session.player_name}</span>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="text-center py-10 text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700/30 rounded-xl border border-dashed border-gray-200 dark:border-gray-600">
                No active playback sessions
            </div>
        {/if}
    </div>
    {:else}
        <div class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
        </div>
    {/if}
</MainLayout>
