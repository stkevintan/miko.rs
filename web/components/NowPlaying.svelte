<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Play } from 'lucide-svelte';
    import DashboardCard from './DashboardCard.svelte';
    import CoverArt from './CoverArt.svelte';
    import { api } from '../lib/api';

    let sessions = $state<any[]>([]);
    let pollInterval: number | null = null;

    async function fetchSessions() {
        try {
            const res = await api.get('/getNowPlaying');
            const data = res.data.nowPlaying;
            if (data && data.entry) {
                // Handle both single object and array of objects
                sessions = Array.isArray(data.entry) ? data.entry : [data.entry];
            } else {
                sessions = [];
            }
        } catch (e) {
            console.error('Failed to fetch now playing', e);
        }
    }

    onMount(() => {
        fetchSessions();
        pollInterval = setInterval(fetchSessions, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });
</script>

<DashboardCard
    title="Now Playing"
    icon={Play}
    class="h-full max-h-[600px] flex flex-col"
>
    {#snippet headerExtra()}
        <span
            class="px-2 py-1 text-xs font-medium bg-orange-100 text-orange-600 rounded-full dark:bg-orange-900/30 dark:text-orange-400"
        >
            {sessions.length} Active
        </span>
    {/snippet}

    <div class="flex-1 overflow-y-auto pr-1 -mr-1 custom-scrollbar">
        {#if sessions.length > 0}
            <div class="space-y-4">
                {#each sessions as session}
                    <div
                        class="flex items-center p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600 hover:border-orange-300 dark:hover:border-orange-500/50 transition-all group"
                    >
                        <CoverArt
                            id={session.coverArt}
                            alt={session.title || ''}
                            size={24}
                            class="w-12 h-12 rounded-lg mr-4 shadow-md group-hover:scale-105 transition-transform"
                        />
                        <div class="min-w-0 flex-1">
                            <p
                                class="text-sm font-bold text-gray-900 dark:text-white truncate"
                            >
                                {session.title || 'Unknown Song'}
                            </p>
                            <p
                                class="text-xs text-gray-500 dark:text-gray-400 truncate"
                            >
                                {session.artist || 'Unknown Artist'}
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
                                    >via {session.playerName}</span
                                >
                                {#if session.minutesAgo > 0}
                                    <span class="ml-2 text-orange-500">
                                        • {session.minutesAgo}m ago
                                    </span>
                                {/if}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {:else}
            <div
                class="flex flex-col items-center justify-center py-12 text-center text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700/30 rounded-xl border border-dashed border-gray-200 dark:border-gray-600 h-full"
            >
                <Play size={32} class="mb-3 opacity-20" />
                <p class="text-sm font-medium">No active playback sessions</p>
            </div>
        {/if}
    </div>
</DashboardCard>
