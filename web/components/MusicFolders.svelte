<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { navigate } from '../router';
    import DashboardCard from './DashboardCard.svelte';
    import { Folder, Pen } from 'lucide-svelte';
    import type { FolderInfo } from '../lib/types';
    import { api } from '../lib/api';
    import { toast } from '../lib/toast.svelte';

    let folders = $state<FolderInfo[]>([]);
    let pollInterval: number | null = null;

    async function fetchFolders() {
        try {
            const res = await api.get('/folders');
            folders = res.data;
        } catch (err) {
            console.error('Failed to fetch folders', err);
            toast.error('Failed to fetch music folders');
        }
    }

    onMount(() => {
        fetchFolders();
        pollInterval = setInterval(fetchFolders, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });
</script>

<DashboardCard
    title="Music Folders"
    icon={Folder}
    class="h-full max-h-[600px] flex flex-col"
>
    {#snippet headerExtra()}
        <button
            type="button"
            onclick={() => navigate('/settings/folders')}
            class="p-1.5 rounded-lg text-gray-400 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-950/30 transition-colors"
            title="Manage Folders"
        >
            <Pen size={20} />
        </button>
    {/snippet}

    <div class="flex-1 overflow-y-auto pr-1 -mr-1 custom-scrollbar">
        {#if folders.length > 0}
            <div class="space-y-4">
                {#each folders as folder}
                    <div
                        class="p-4 rounded-xl bg-gray-50 dark:bg-gray-700/50 border border-gray-100 dark:border-gray-600"
                    >
                        <div class="flex justify-between items-start mb-2">
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
        {:else}
            <button
                type="button"
                onclick={() => navigate('/settings/folders')}
                class="flex flex-col items-center justify-center py-12 text-center text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700/30 rounded-xl border border-dashed border-gray-200 dark:border-gray-600 h-full w-full hover:bg-gray-100 dark:hover:bg-gray-700/50 hover:border-orange-300 dark:hover:border-orange-500/50 transition-all group"
            >
                <Folder size={32} class="mb-3 opacity-20 group-hover:opacity-40 group-hover:scale-110 transition-all" />
                <p class="text-sm font-medium group-hover:text-orange-600 dark:group-hover:text-orange-400 transition-colors">No music folders configured</p>
                <p class="text-xs opacity-60 mt-1 px-4">
                    Add folders in settings to start scanning your library.
                </p>
            </button>
        {/if}
    </div>
</DashboardCard>
