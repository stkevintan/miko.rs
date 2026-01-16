<script lang="ts">
    import { Music } from 'lucide-svelte';
    import { getCoverArtUrl } from '../lib/api';

    type IconComponent = typeof import('lucide-svelte')['Music'];

    /**
     * CoverArt component for displaying music imagery with fallbacks.
     */
    let {
        id,
        alt = '',
        size = 24,
        class: className = '',
        fallbackClass = 'bg-orange-500',
        icon = Music,
    } = $props<{
        id?: string | null;
        alt?: string;
        size?: number;
        class?: string;
        fallbackClass?: string;
        icon?: IconComponent;
    }>();

    let imageLoaded = $state(false);
    let imageError = $state(false);
    let imageUrl = $state('');

    // Reset state when ID changes
    $effect(() => {
        if (!id) {
            imageUrl = '';
            return;
        }

        const controller = new AbortController();

        async function fetchImage() {
            try {
                const url = await getCoverArtUrl(id, controller.signal);
                if (imageUrl) URL.revokeObjectURL(imageUrl);
                imageUrl = url;
                imageLoaded = true;
            } catch (err: any) {
                if (err instanceof Error && err.name !== 'AbortError') {
                    imageError = true;
                }
            }
        }

        fetchImage();

        return () => {
            controller.abort();
        };
    });
</script>

<div
    class="relative flex items-center justify-center overflow-hidden text-white shrink-0 {fallbackClass} {className}"
>
    {#if icon}
        {@const Icon = icon}
        <Icon {size} />
    {/if}

    {#if id && imageUrl}
        <img
            src={imageUrl}
            {alt}
            class="absolute inset-0 w-full h-full object-cover transition-opacity duration-300 {imageLoaded
                ? 'opacity-100'
                : 'opacity-0'}"
            onload={() => {
                imageLoaded = true;
                imageError = false;
            }}
            onerror={() => {
                imageError = true;
                imageLoaded = false;
            }}
            style:display={imageError ? 'none' : 'block'}
        />
    {/if}
</div>
