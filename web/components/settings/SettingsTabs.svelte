<script lang="ts">
    import { route } from '../../router';
    import { User, Folder, Plug, Users } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import { authStore } from '../../lib/auth.svelte';

    onMount(() => {
        authStore.fetchProfile();
    });

    const tabs = [
        { label: 'Profile', path: '/settings/profile', icon: User, adminOnly: false },
        { label: 'Folders', path: '/settings/folders', icon: Folder, adminOnly: false },
        { label: 'Connections', path: '/settings/connections', icon: Plug, adminOnly: false },
        { label: 'Users', path: '/settings/users', icon: Users, adminOnly: true },
    ];

    const isActive = (path: string) => (route.pathname || '').startsWith(path);
</script>

<div
    class="inline-flex rounded-xl bg-gray-100 dark:bg-gray-800 p-1 border border-gray-200 dark:border-gray-700"
    role="tablist"
>
    {#each tabs as tab}
        {#if !tab.adminOnly || authStore.user?.adminRole}
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
        {/if}
    {/each}
</div>
