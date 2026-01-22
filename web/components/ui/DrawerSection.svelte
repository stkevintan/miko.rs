<script lang="ts">
    import type { Snippet } from 'svelte';

    let { 
        title, 
        icon: IconSnippet, 
        children,
        action: ActionSnippet
    }: {
        title: string;
        icon?: Snippet;
        children: Snippet;
        action?: Snippet;
    } = $props();
</script>

<section>
    <h4 class="text-xs font-bold text-gray-400 uppercase tracking-widest mb-4 flex items-center gap-2">
        {#if IconSnippet}
            <span class="flex-shrink-0">
                {@render IconSnippet()}
            </span>
        {/if}
        <span class="flex-1 truncate">{title}</span>
        {#if ActionSnippet}
            {@render ActionSnippet()}
        {/if}
    </h4>
    {@render children()}
</section>
