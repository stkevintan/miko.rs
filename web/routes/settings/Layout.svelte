<script lang="ts">
    import type { Snippet } from 'svelte';

    let { children } = $props<{
        children: Snippet;
    }>();
</script>

<div class="flex flex-col h-[calc(100vh-8rem)]">
    {@render children()}
</div>

<style lang="postcss">
    @reference "../../style.css";
</style>
