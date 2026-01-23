<script lang="ts">
    import { X } from 'lucide-svelte';
    import type { Snippet } from 'svelte';

    let { 
        title, 
        subtitle, 
        icon: IconSnippet, 
        action: ActionSnippet,
        onClose 
    }: {
        title: string;
        subtitle?: string;
        icon?: Snippet;
        action?: Snippet;
        onClose: () => void;
    } = $props();
</script>

<div class="p-6 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between bg-gray-50 dark:bg-gray-900/50 flex-shrink-0">
    <div class="flex items-center gap-3 min-w-0 flex-1">
        {#if IconSnippet}
            <div class="p-2 bg-orange-100 dark:bg-orange-900/30 rounded-lg text-orange-600 flex-shrink-0">
                {@render IconSnippet()}
            </div>
        {/if}
        <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
                <h3 class="text-lg font-bold text-gray-900 dark:text-white truncate">
                    {title}
                </h3>
                {#if ActionSnippet}
                    {@render ActionSnippet()}
                {/if}
            </div>
            {#if subtitle}
                <p class="text-xs text-gray-500 truncate">
                    {subtitle}
                </p>
            {/if}
        </div>
    </div>
    <button 
        onclick={onClose} 
        class="p-2 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors flex-shrink-0"
        aria-label="Close drawer"
    >
        <X size={20} />
    </button>
</div>
