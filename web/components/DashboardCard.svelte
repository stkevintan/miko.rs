<script lang="ts">
    import type { Snippet } from 'svelte';

    let { 
        title, 
        icon: Icon, 
        iconClass = "text-orange-600",
        headerExtra,
        children,
        class: className = ""
    } = $props<{
        title: string;
        icon?: any;
        iconClass?: string;
        headerExtra?: Snippet;
        children?: Snippet;
        class?: string;
    }>();
</script>

<div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700 @container {className}">
    <div class="flex items-center justify-between mb-6 shrink-0">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center">
            {#if Icon}
                <Icon size={20} class="mr-2 {iconClass}" />
            {/if}
            {title}
        </h2>
        {#if headerExtra}
            {@render headerExtra()}
        {/if}
    </div>
    
    {#if children}
        {@render children()}
    {/if}
</div>
