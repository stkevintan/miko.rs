<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { onMount, onDestroy } from 'svelte';
    import MainLayout from '../components/MainLayout.svelte';
    import StatCard from '../components/StatCard.svelte';
    import { authStore } from '../lib/auth.svelte';
    import api from '../lib/api';
    import type { DashboardData } from '../lib/types';
    import { Music, User, Cpu, Play, Disc, List, Tag, Folder, Zap, RefreshCw, Loader2 } from 'lucide-svelte';
    import Dropdown from '../components/ui/Dropdown.svelte';

    let token = $state(localStorage.getItem('token'));
    let data = $state<DashboardData | null>(null);
    let pollInterval: number | null = null;

    async function fetchData() {
        try {
            const [stats, system, folders, nowPlaying, scanStatus] = await Promise.all([
                api.get('/stats'),
                api.get('/system'),
                api.get('/folders'),
                api.get('/now-playing'),
                api.get('/scan')
            ]);
            data = {
                stats: stats.data,
                system: system.data,
                folders: folders.data,
                now_playing: nowPlaying.data,
                scan_status: scanStatus.data
            };
        } catch (e) {
            console.error('Failed to fetch dashboard data', e);
        }
    }

    async function startScan(full = false) {
        if (data?.scan_status.scanning) return;
        try {
            await api.post(`/scan?full=${full}`);
            await fetchData();
        } catch (e) {
            console.error('Failed to start scan', e);
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
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Dashboard</h1>
        
        {#if data}
            <div class="relative">
                <Dropdown triggerMode="hover" align="right">
                    {#snippet trigger()}
                        <button 
                            class="flex cursor-pointer items-center px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg transition-colors shadow-sm disabled:cursor-not-allowed min-w-[120px] justify-center relative"
                            onclick={() => startScan(false)}
                            disabled={data?.scan_status.scanning}
                        >
                            {#if data?.scan_status.scanning}
                                <Loader2 size={18} class="mr-2 animate-spin text-orange-500" />
                                <span class="text-orange-600 dark:text-orange-400 font-medium">{data.scan_status.count}/{data.scan_status.total}</span>
                                <div class="absolute -top-1 -right-1 flex h-3 w-3">
                                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75"></span>
                                    <span class="relative inline-flex rounded-full h-3 w-3 bg-orange-500"></span>
                                </div>
                            {:else}
                                <RefreshCw size={18} class="mr-2 text-gray-400" />
                                <span>Scan</span>
                            {/if}
                        </button>
                    {/snippet}
                    {#snippet content()}
                        <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl overflow-hidden py-1 w-48">
                            <div class="px-4 py-2 text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider border-b border-gray-100 dark:border-gray-700 mb-1">
                                Library Scan
                            </div>
                            <button 
                                class="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
                                onclick={() => startScan(false)}
                                disabled={data?.scan_status.scanning}
                            >
                                <Zap size={14} class="mr-2 text-yellow-500" />
                                Quick Scan
                            </button>
                            <button 
                                class="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
                                onclick={() => startScan(true)}
                                disabled={data?.scan_status.scanning}
                            >
                                <RefreshCw size={14} class="mr-2 text-blue-500" />
                                Full Scan
                            </button>
                        </div>
                    {/snippet}
                </Dropdown>
            </div>
        {/if}
    </div>

    {#if data}
        <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-6 mb-8">
            <StatCard label="Songs" value={data.stats.songs} color="orange" icon={Music} />
            <StatCard label="Albums" value={data.stats.albums} color="blue" icon={Disc} />
            <StatCard label="Artists" value={data.stats.artists} color="green" icon={User} />
            <StatCard label="Genres" value={data.stats.genres} color="red" icon={Tag} />
            <StatCard label="Playlists" value={data.stats.playlists} color="purple" icon={List} />
        </div>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- First Column: User Profile + Now Playing -->
        <div class="space-y-4">
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
                    <div class="space-y-4">
                        {#each data.now_playing as session}
                            <div class="flex items-center p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600 hover:border-orange-300 dark:hover:border-orange-500/50 transition-all group">
                                <div class="w-12 h-12 rounded-lg bg-orange-500 flex items-center justify-center text-white mr-4 shrink-0 shadow-md group-hover:scale-105 transition-transform overflow-hidden relative">
                                    <Music size={24} />
                                    {#if session.cover_art}
                                        {#key session.cover_art}
                                            <img 
                                                src="/api/coverart/{session.cover_art}?token={token}" 
                                                alt={session.song_title}
                                                class="absolute inset-0 w-full h-full object-cover"
                                                onload={(e) => (e.currentTarget as HTMLImageElement).style.display = 'block'}
                                                onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'}
                                            />
                                        {/key}
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
        </div>

        <!-- Second Column: App Resources + Music Folders -->
        <div class="space-y-4">
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

            <!-- Music Folders section -->
            <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center mb-6">
                    <Folder size={20} class="mr-2 text-orange-600" />
                    Music Folders
                </h2>
                
                <div class="space-y-6">
                    {#each data.folders as folder}
                        <div class="p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600">
                            <div class="flex justify-between items-start mb-2">
                                <div class="min-w-0 flex-1">
                                    <p class="text-sm font-bold text-gray-900 dark:text-white truncate">
                                        {folder.label}
                                    </p>
                                    <p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-1" title={folder.path}>
                                        {folder.path}
                                    </p>
                                </div>
                                <div class="ml-4 text-right">
                                    <span class="px-2.5 py-1 text-xs font-bold bg-orange-100 text-orange-600 rounded-full dark:bg-orange-900/40 dark:text-orange-400">
                                        {folder.song_count.toLocaleString()} songs
                                    </span>
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        </div>
    </div>
    {:else}
        <div class="flex items-center justify-center h-64">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
        </div>
    {/if}
</MainLayout>
