<script lang="ts">
    import type { Snippet } from 'svelte';
    import LibraryTabs from '../../components/library/LibraryTabs.svelte';

    let { children } = $props<{
        children: Snippet;
    }>();
</script>

<div class="flex flex-col h-[calc(100vh-8rem)]">
    <div class="flex flex-col md:flex-row md:items-center mb-4 gap-4">
        <h1 class="text-3xl mr-auto font-extrabold text-gray-900 dark:text-white">
            Library
        </h1>
        <LibraryTabs />
    </div>

    {@render children()}
</div>

<style lang="postcss">
    @reference "../../style.css";
</style>
