<script lang="ts">
    import { route } from '../../router';
    import { Music, Disc, User, Tag } from 'lucide-svelte';

    const tabs = [
        { label: 'Tracks', path: '/library/tracks', icon: Music },
        { label: 'Albums', path: '/library/albums', icon: Disc },
        { label: 'Artists', path: '/library/artists', icon: User },
        { label: 'Genres', path: '/library/genres', icon: Tag },
    ];

    const isActive = (path: string) => (route.pathname || '').startsWith(path);
</script>

<div
    class="inline-flex rounded-xl bg-gray-100 dark:bg-gray-800 p-1 border border-gray-200 dark:border-gray-700"
    role="tablist"
>
    {#each tabs as tab}
        <a
            href={tab.path}
            class="px-4 py-2 text-sm font-semibold rounded-lg transition-colors flex items-center
            {isActive(tab.path)
                ? 'bg-white dark:bg-gray-900 text-gray-900 dark:text-white shadow'
                : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'}"
            role="tab"
            aria-selected={isActive(tab.path)}
        >
            <tab.icon size={16} class="mr-2" />
            {tab.label}
        </a>
    {/each}
</div>
