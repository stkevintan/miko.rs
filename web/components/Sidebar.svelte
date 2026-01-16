<script lang="ts">
    import {
        Home,
        Library,
        Settings,
        X,
        ChevronDown,
        ChevronRight,
        Music,
        Disc,
        User,
        Tag,
        Folder,
        Globe,
        Users,
    } from 'lucide-svelte';
    import { isActive2 } from '../router';

    let { isOpen = true, onToggle } = $props<{
        isOpen?: boolean;
        onToggle?: () => void;
    }>();

    let libraryExpanded = $state(isActive2('/library'));
    let settingsExpanded = $state(isActive2('/settings'));

    $effect(() => {
        if (isActive2('/library')) {
            libraryExpanded = true;
        }
    });

    $effect(() => {
        if (isActive2('/settings')) {
            settingsExpanded = true;
        }
    });

    const navItems = [
        { name: 'Home', path: '/dashboard', catalog: '/dashboard', icon: Home },
    ];

    const libraryItems = [
        { name: 'Tracks', path: '/library/tracks', icon: Music },
        { name: 'Albums', path: '/library/albums', icon: Disc },
        { name: 'Artists', path: '/library/artists', icon: User },
        { name: 'Genres', path: '/library/genres', icon: Tag },
    ];

    const settingsItems = [
        { name: 'Profile', path: '/settings/profile', icon: User },
        { name: 'Folders', path: '/settings/folders', icon: Folder },
        { name: 'Connections', path: '/settings/connections', icon: Globe },
        { name: 'Users', path: '/settings/users', icon: Users },
    ];
</script>

<!-- Mobile Overlay -->
{#if isOpen}
    <button
        class="fixed inset-0 bg-black/50 z-20 transition-opacity lg:hidden w-full h-full cursor-default"
        onclick={onToggle}
        aria-label="Close sidebar"
    ></button>
{/if}

<aside
    class="fixed top-0 left-0 z-30 h-screen transition-transform bg-white border-r border-gray-200 dark:bg-gray-800 dark:border-gray-700
  {isOpen
        ? 'w-64 translate-x-0'
        : 'w-0 -translate-x-full lg:w-20 lg:translate-x-0'} overflow-hidden"
>
    <div class="flex flex-col h-full">
        <div class="flex items-center justify-between h-16 px-4 border-b">
            <span
                class="text-xl font-bold text-orange-600 {isOpen
                    ? 'block'
                    : 'hidden lg:hidden'}">Miko</span
            >
            <button
                class="p-2 rounded-lg hover:bg-gray-100 lg:hidden"
                onclick={onToggle}
            >
                <X size={20} />
            </button>
        </div>

        <nav class="flex-1 px-3 py-4 space-y-1">
            {#each navItems as item}
                <a
                    href={item.path}
                    onclick={(e) => {
                        if (isActive2(item.catalog)) {
                            e.preventDefault();
                            e.stopPropagation();
                        }
                    }}
                    class="flex items-center p-2 text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group
          {isActive2(item.catalog)
                        ? 'bg-orange-50 text-orange-600 dark:bg-gray-700'
                        : ''}"
                >
                    <item.icon
                        size={20}
                        class={isActive2(item.catalog)
                            ? 'text-orange-600'
                            : 'text-gray-500'}
                    />
                    <span
                        class="ms-3 transition-opacity duration-300 {isOpen
                            ? 'opacity-100'
                            : 'opacity-0 lg:hidden'}">{item.name}</span
                    >
                </a>
            {/each}

            <!-- Expandable Library -->
            <div class="space-y-1">
                <button
                    onclick={() => libraryExpanded = !libraryExpanded}
                    class="flex items-center w-full p-2 text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group
                        {isActive2('/library') ? 'text-orange-600' : ''}"
                >
                    <Library
                        size={20}
                        class={isActive2('/library') ? 'text-orange-600' : 'text-gray-500'}
                    />
                    <span class="ms-3 transition-opacity duration-300 flex-1 text-left {isOpen ? 'opacity-100' : 'opacity-0 lg:hidden'}">
                        Library
                    </span>
                    {#if isOpen}
                        {#if libraryExpanded}
                            <ChevronDown size={16} class="text-gray-400" />
                        {:else}
                            <ChevronRight size={16} class="text-gray-400" />
                        {/if}
                    {/if}
                </button>

                {#if libraryExpanded && isOpen}
                    <div class="space-y-1 ms-4 mt-1">
                        {#each libraryItems as subItem}
                            <a
                                href={subItem.path}
                                class="flex items-center p-2 text-sm text-gray-600 rounded-lg dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 group
                                    {isActive2(subItem.path) ? 'text-orange-600 bg-orange-50 dark:bg-gray-700 font-medium' : ''}"
                            >
                                <subItem.icon size={16} class="me-3" />
                                {subItem.name}
                            </a>
                        {/each}
                    </div>
                {/if}
            </div>

            <div class="space-y-1">
                <button
                    onclick={() => (settingsExpanded = !settingsExpanded)}
                    class="flex items-center w-full p-2 text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group
                        {isActive2('/settings') ? 'text-orange-600' : ''}"
                >
                    <Settings
                        size={20}
                        class={isActive2('/settings')
                            ? 'text-orange-600'
                            : 'text-gray-500'}
                    />
                    <span
                        class="ms-3 transition-opacity duration-300 flex-1 text-left {isOpen
                            ? 'opacity-100'
                            : 'opacity-0 lg:hidden'}"
                    >
                        Settings
                    </span>
                    {#if isOpen}
                        {#if settingsExpanded}
                            <ChevronDown size={16} class="text-gray-400" />
                        {:else}
                            <ChevronRight size={16} class="text-gray-400" />
                        {/if}
                    {/if}
                </button>

                {#if settingsExpanded && isOpen}
                    <div class="space-y-1 ms-4 mt-1">
                        {#each settingsItems as subItem}
                            <a
                                href={subItem.path}
                                class="flex items-center p-2 text-sm text-gray-600 rounded-lg dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 group
                                    {isActive2(subItem.path)
                                    ? 'text-orange-600 bg-orange-50 dark:bg-gray-700 font-medium'
                                    : ''}"
                            >
                                <subItem.icon size={16} class="me-3" />
                                {subItem.name}
                            </a>
                        {/each}
                    </div>
                {/if}
            </div>
        </nav>
    </div>
</aside>
