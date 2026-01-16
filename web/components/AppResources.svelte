<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Cpu } from 'lucide-svelte';
    import DashboardCard from './DashboardCard.svelte';
    import { api } from '../lib/api';
    import type { SystemInfo } from '../lib/types';
    import { toast } from '../lib/toast.svelte';

    let system = $state<SystemInfo | null>(null);
    let pollInterval: number | null = null;

    async function fetchSystemInfo() {
        try {
            const res = await api.get('/system');
            system = res.data;
        } catch (err) {
            console.error('Failed to fetch system info', err);
            toast.error('Failed to fetch system information');
            system = {
                cpu_usage: 0,
                memory_usage: 0,
                memory_total: 0,
            };
        }
    }

    onMount(() => {
        fetchSystemInfo();
        pollInterval = setInterval(fetchSystemInfo, 5000);
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

<DashboardCard
    title="App Resources"
    icon={Cpu}
    iconClass="text-blue-600"
    class="h-full"
>
    {#if system}
        <div class="space-y-6">
            <div>
                <div class="flex justify-between mb-2">
                    <span
                        class="text-sm font-medium text-gray-700 dark:text-gray-300"
                    >
                        CPU Usage
                    </span>
                    <span
                        class="text-sm font-bold text-blue-600 dark:text-blue-400"
                    >
                        {system.cpu_usage.toFixed(1)}%
                    </span>
                </div>
                <div
                    class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden"
                >
                    <div
                        class="bg-blue-600 h-2.5 rounded-full transition-all duration-500"
                        style="width: {Math.min(system.cpu_usage, 100)}%"
                    ></div>
                </div>
            </div>

            <div>
                <div class="flex justify-between mb-2">
                    <span
                        class="text-sm font-medium text-gray-700 dark:text-gray-300"
                    >
                        Memory Usage
                    </span>
                    <span
                        class="text-sm font-bold text-purple-600 dark:text-purple-400"
                    >
                        {formatBytes(system.memory_usage)}
                    </span>
                </div>
                <div
                    class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700 overflow-hidden"
                >
                    <div
                        class="bg-purple-600 h-2.5 rounded-full transition-all duration-500"
                        style="width: {(system.memory_usage /
                            system.memory_total) *
                            100}%"
                    ></div>
                </div>
                <p class="text-[10px] text-gray-400 mt-1 text-right">
                    of {formatBytes(system.memory_total)} total system RAM
                </p>
            </div>
        </div>
    {:else}
        <div class="flex items-center justify-center h-full">
            <div
                class="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"
            ></div>
        </div>
    {/if}
</DashboardCard>
