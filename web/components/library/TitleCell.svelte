<script lang="ts">
    import { Play } from 'lucide-svelte';
    import CoverArt from '../CoverArt.svelte';

    let {
        title,
        coverArt,
        showPlay = true,
    }: {
        title: string;
        coverArt?: string | null;
        showPlay?: boolean;
    } = $props();
</script>

<div class="flex items-center gap-3 py-1 min-w-0 overflow-hidden">
    <div class="relative group/cover transition-shadow shrink-0">
        <CoverArt 
            id={coverArt} 
            size={14} 
            class="w-10 h-10 rounded-lg" 
            fallbackClass="bg-gray-100 dark:bg-gray-800 text-gray-400" 
        />
        {#if showPlay}
            <div class="absolute inset-0 bg-black/20 opacity-0 group-hover/cover:opacity-100 flex items-center justify-center transition-opacity rounded-lg">
                <Play size={16} class="text-white fill-current" />
            </div>
        {/if}
    </div>
    <div class="min-w-0">
        <div class="text-sm font-semibold text-gray-900 dark:text-white truncate">{title}</div>
    </div>
</div>
