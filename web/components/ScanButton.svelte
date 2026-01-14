<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { RefreshCw, Loader2, Zap } from 'lucide-svelte';
    import Dropdown from './ui/Dropdown.svelte';
    import { subsonic } from '../lib/api';
    import type { ScanStatus } from '../lib/types';

    let scanStatus = $state<ScanStatus | null>(null);
    let pollInterval: number | null = null;

    async function fetchScanStatus() {
        try {
            const res = await subsonic.get('/getScanStatus');
            const data = res.data.scanStatus;
            // Map Subsonic ScanStatus to our ScanStatus type
            scanStatus = {
                scanning: data.scanning,
                count: data.count || 0,
                total: data.total || 0
            };
        } catch (e) {
            console.error('Failed to fetch scan status', e);
        }
    }

    async function startScan(full = false) {
        if (scanStatus?.scanning) return;
        try {
            await subsonic.get('/startScan', {
                params: {
                    fullScan: full
                }
            });
            await fetchScanStatus();
        } catch (e) {
            console.error('Failed to start scan', e);
        }
    }

    onMount(() => {
        fetchScanStatus();
        pollInterval = setInterval(fetchScanStatus, 5000);
    });

    onDestroy(() => {
        if (pollInterval) clearInterval(pollInterval);
    });
</script>

<div class="relative">
    <Dropdown triggerMode="hover" align="right">
        {#snippet trigger()}
            <button 
                class="flex cursor-pointer items-center p-2 text-gray-500 rounded-lg hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 transition-colors disabled:cursor-not-allowed relative group"
                onclick={() => startScan(false)}
                disabled={scanStatus?.scanning}
            >
                {#if scanStatus?.scanning}
                    <Loader2 size={20} class="animate-spin text-orange-500" />
                    <span class="ml-2 text-xs font-medium text-orange-600 dark:text-orange-400">
                        {scanStatus.count}/{scanStatus.total}
                    </span>
                    <div class="absolute -top-1 -right-1 flex h-3 w-3">
                        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75"></span>
                        <span class="relative inline-flex rounded-full h-3 w-3 bg-orange-500"></span>
                    </div>
                {:else}
                    <RefreshCw size={20} class="group-hover:rotate-180 transition-transform duration-500" />
                {/if}
            </button>
        {/snippet}
        {#snippet content()}
            <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl overflow-hidden py-1 w-48">
                <div class="px-4 py-2 text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider border-b border-gray-100 dark:border-gray-700 mb-1">
                    Library Scan
                </div>
                <button 
                    class="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 cursor-pointer"
                    onclick={() => startScan(false)}
                    disabled={scanStatus?.scanning}
                >
                    <Zap size={14} class="mr-2 text-yellow-500" />
                    Quick Scan
                </button>
                <button 
                    class="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 cursor-pointer"
                    onclick={() => startScan(true)}
                    disabled={scanStatus?.scanning}
                >
                    <RefreshCw size={14} class="mr-2 text-blue-500" />
                    Full Scan
                </button>
            </div>
        {/snippet}
    </Dropdown>
</div>
