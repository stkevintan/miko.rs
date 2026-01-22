<script lang="ts">
    import type { Snippet } from 'svelte';

    let { 
        title, 
        icon: IconSnippet, 
        children 
    }: {
        title: string;
        icon?: Snippet;
        children: Snippet;
    } = $props();
</script>

<section>
    <h4 class="text-xs font-bold text-gray-400 uppercase tracking-widest mb-4 flex items-center gap-2">
        {#if IconSnippet}
            {@render IconSnippet()}
        {/if}
        {title}
    </h4>
    {@render children()}
</section>
