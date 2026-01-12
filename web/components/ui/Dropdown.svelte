<script lang="ts">
    import { type Snippet } from 'svelte';
    import { fade } from 'svelte/transition';

    let {
        trigger,
        content,
        align = 'auto',
        triggerMode = 'click',
    } = $props<{
        trigger: Snippet;
        content: Snippet;
        align?: 'left' | 'right' | 'middle' | 'auto';
        triggerMode?: 'click' | 'hover';
    }>();

    let isOpen = $state(false);
    let containerEl = $state<HTMLElement | null>(null);
    let calculatedAlign = $state<'left' | 'right' | 'middle'>('middle');

    $effect(() => {
        if (align !== 'auto') {
            calculatedAlign = align;
            return;
        }

        if (isOpen && containerEl) {
            const rect = containerEl.getBoundingClientRect();
            const viewportWidth = window.innerWidth;
            const dropdownWidth = 192; // w-48 is 12rem = 192px

            const fitsMiddle = rect.left + rect.width / 2 - dropdownWidth / 2 >= 0 && 
                               rect.left + rect.width / 2 + dropdownWidth / 2 <= viewportWidth;
            
            if (fitsMiddle) {
                calculatedAlign = 'middle';
            } else if (rect.left + rect.width / 2 > viewportWidth / 2) {
                calculatedAlign = 'right';
            } else {
                calculatedAlign = 'left';
            }
        }
    });

    function toggle() {
        if (triggerMode === 'click') {
            isOpen = !isOpen;
        }
    }

    function open() {
        if (triggerMode === 'hover') {
            isOpen = true;
        }
    }

    function close() {
        isOpen = false;
    }
</script>

<div 
    bind:this={containerEl}
    class="relative group" 
    role="group" 
    onmouseenter={open} 
    onmouseleave={close}
>
    <!-- Trigger -->
    <div
        onclick={toggle}
        onkeydown={(e) => e.key === 'Enter' && toggle()}
        role="button"
        tabindex="0"
        class="cursor-pointer"
    >
        {@render trigger()}
    </div>

    <!-- Overlay for outside click (only for click mode) -->
    {#if isOpen && triggerMode === 'click'}
        <button
            class="fixed inset-0 z-40 w-full h-full cursor-default"
            onclick={close}
            aria-label="Close menu"
        ></button>
    {/if}

    <!-- Dropdown Content -->
    {#if isOpen}
        <div
            transition:fade={{ duration: 100 }}
            class="absolute z-50 mt-0 pt-2 w-48 {calculatedAlign === 'right'
                ? 'right-0'
                : calculatedAlign === 'left'
                  ? 'left-0'
                  : 'left-1/2 -translate-x-1/2'}"
            role="presentation"
            onmouseenter={open}
        >
            <div
                class="bg-white divide-y divide-gray-100 rounded shadow-lg border border-gray-100 dark:bg-gray-700 dark:divide-gray-600 dark:border-gray-600 overflow-hidden"
                role="menu"
                tabindex="-1"
                onclick={close}
                onkeydown={(e) => (e.key === 'Escape' || e.key === 'Enter') && close()}
            >
                {@render content()}
            </div>
        </div>
    {/if}
</div>
