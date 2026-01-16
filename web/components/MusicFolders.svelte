<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import DashboardCard from './DashboardCard.svelte';
    import { Folder } from 'lucide-svelte';
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
    <div class="flex-1 overflow-y-auto pr-1 -mr-1 custom-scrollbar">
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
    </div>
</DashboardCard>
