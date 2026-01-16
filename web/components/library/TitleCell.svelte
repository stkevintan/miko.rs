<script lang="ts">
    import { Play } from 'lucide-svelte';
    import type { Song } from '../../lib/types';
    import CoverArt from '../CoverArt.svelte';

    let { song }: { song: Song } = $props();
</script>

<div class="flex items-center gap-3 py-1 min-w-0 overflow-hidden">
    <div class="relative group/cover transition-shadow shrink-0">
        <CoverArt 
            id={song.coverArt} 
            size={14} 
            class="w-10 h-10 rounded-lg" 
            fallbackClass="bg-gray-100 dark:bg-gray-800 text-gray-400" 
        />
        <div class="absolute inset-0 bg-black/20 opacity-0 group-hover/cover:opacity-100 flex items-center justify-center transition-opacity rounded-lg">
            <Play size={16} class="text-white fill-current" />
        </div>
    </div>
    <div class="min-w-0">
        <div class="text-sm font-semibold text-gray-900 dark:text-white truncate">{song.title}</div>
    </div>
</div>
