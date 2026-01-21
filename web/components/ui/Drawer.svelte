<script lang="ts">
    import { fade, fly } from 'svelte/transition';

    let { 
        isOpen = $bindable(false), 
        width = "450px",
        children
    } = $props();

    function close() {
        isOpen = false;
    }

    $effect(() => {
        if (isOpen) {
            document.body.style.overflow = 'hidden';
            return () => {
                document.body.style.overflow = '';
            };
        }
    });
</script>

{#if isOpen}
    <div 
        class="fixed inset-0 z-40 bg-black/20 backdrop-blur-sm transition-opacity"
        onclick={close}
        aria-hidden="true"
        transition:fade={{ duration: 200 }}
    ></div>
    <div 
        class="fixed right-0 top-0 h-full bg-white dark:bg-gray-800 shadow-2xl z-50 flex flex-col w-full sm:w-[var(--drawer-width)] sm:max-w-[calc(100vw-2rem)]"
        style="--drawer-width: {width};"
        transition:fly={{ x: 1000, duration: 300 }}
    >
        {@render children()}
    </div>
{/if}
