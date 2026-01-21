<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import type { FolderInfo } from '../../lib/types';
    import { authStore } from '../../lib/auth.svelte';
    import { Plus, Pencil, Trash2, X } from 'lucide-svelte';

    let folders = $state<FolderInfo[]>([]);
    let loading = $state(false);
    let showModal = $state(false);
    let editingFolder = $state<FolderInfo | null>(null);

    let folderPath = $state('');
    let folderName = $state('');

    async function fetchFolders() {
        loading = true;
        try {
            const response = await api.get<FolderInfo[]>('/folders');
            folders = response.data;
        } catch (error) {
            console.error('Failed to fetch folders:', error);
            toast.error('Failed to load music folders');
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        authStore.fetchProfile();
        fetchFolders();
    });

    function openAddModal() {
        editingFolder = null;
        folderPath = '';
        folderName = '';
        showModal = true;
    }

    function openEditModal(folder: FolderInfo) {
        editingFolder = folder;
        folderPath = folder.path;
        folderName = folder.label;
        showModal = true;
    }

    async function handleSave() {
        if (!folderPath) {
            toast.error('Folder path is required');
            return;
        }

        try {
            if (editingFolder) {
                await api.post(`/folders/${editingFolder.id}`, {
                    path: folderPath,
                    name: folderName,
                });
                toast.add('Folder updated successfully', 'success');
            } else {
                await api.post('/folders', {
                    path: folderPath,
                    name: folderName,
                });
                toast.add('Folder added successfully', 'success');
            }
            showModal = false;
            fetchFolders();
            // Trigger a quick scan
            api.get('/startScan').catch((err) => {
                toast.error('Failed to trigger scan after saving folder');
                console.error('Failed to trigger scan', err);
            });
        } catch (error: any) {
            toast.error(error.response?.data?.error || 'Failed to save folder');
        }
    }

    async function handleDelete(id: number) {
        if (
            !confirm(
                'Are you sure you want to remove this music folder? Songs in this folder will be removed from the library (but files on disk will be kept).',
            )
        ) {
            return;
        }

        try {
            await api.delete(`/folders/${id}`);
            toast.add('Folder removed successfully', 'success');
            fetchFolders();
            // Trigger a quick scan to clean up removed folders/files
            api.get('/startScan').catch((err) =>
                console.error('Failed to trigger scan', err),
            );
        } catch (error: any) {
            toast.error(
                error.response?.data?.error || 'Failed to delete folder',
            );
        }
    }
</script>

<div class="flex items-center mb-4 gap-6">
    <div class="mr-auto flex items-center gap-3">
        <h2
            class="text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
        >
            Folders
        </h2>
    </div>
    {#if authStore.user?.adminRole}
        <button
            type="button"
            class="flex items-center gap-2 px-3 py-2 rounded-lg bg-orange-600 text-white text-sm font-semibold hover:bg-orange-700 transition-colors"
            onclick={openAddModal}
        >
            <Plus size={16} />
            Add Folder
        </button>
    {/if}
</div>

<div
    class="bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm overflow-hidden"
>
    <div
        class="p-4 border-b border-gray-100 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-800/50"
    >
        <p class="text-sm text-gray-500 dark:text-gray-400">
            {#if authStore.user?.adminRole}
                Manage the root directories where your music is stored.
            {:else}
                You can view the music library source folders. Admin access is
                required to make changes.
            {/if}
        </p>
    </div>
    <div class="overflow-x-auto">
        <table class="min-w-full text-sm">
            <thead
                class="text-left text-gray-500 dark:text-gray-400 bg-gray-50/30 dark:bg-gray-800/30"
            >
                <tr>
                    <th
                        class="px-4 py-3 font-semibold text-xs uppercase tracking-wider"
                        >Name</th
                    >
                    <th
                        class="px-4 py-3 font-semibold text-xs uppercase tracking-wider"
                        >Path</th
                    >
                    <th
                        class="px-4 py-3 font-semibold text-xs uppercase tracking-wider text-right"
                        >Songs</th
                    >
                    <th
                        class="px-4 py-3 font-semibold text-xs uppercase tracking-wider text-right"
                        >Actions</th
                    >
                </tr>
            </thead>
            <tbody class="divide-y divide-gray-100 dark:divide-gray-800">
                {#if loading && folders.length === 0}
                    <tr>
                        <td class="px-6 py-12 text-center" colspan="4">
                            <div class="flex flex-col items-center gap-2">
                                <div
                                    class="animate-spin rounded-full h-6 w-6 border-b-2 border-orange-500"
                                ></div>
                                <span class="text-gray-500"
                                    >Loading music folders...</span
                                >
                            </div>
                        </td>
                    </tr>
                {:else if folders.length === 0}
                    <tr>
                        <td
                            class="px-6 py-12 text-center text-gray-500 dark:text-gray-400"
                            colspan="4"
                        >
                            No music folders configured.
                        </td>
                    </tr>
                {:else}
                    {#each folders as folder}
                        <tr
                            class="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors odd:bg-white even:bg-gray-50/50 dark:odd:bg-gray-900 dark:even:bg-gray-800/30"
                        >
                            <td class="px-4 py-3">
                                <span
                                    class="font-medium text-gray-900 dark:text-white"
                                    >{folder.label}</span
                                >
                            </td>
                            <td class="px-4 py-3">
                                <code
                                    class="text-xs bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-gray-600 dark:text-gray-400"
                                >
                                    {folder.path}
                                </code>
                            </td>
                            <td
                                class="px-4 py-3 text-right text-gray-500 dark:text-gray-400"
                            >
                                {folder.song_count.toLocaleString()}
                            </td>
                            <td class="px-4 py-3 text-right">
                                <div class="flex justify-end gap-2">
                                    {#if authStore.user?.adminRole}
                                        <button
                                            type="button"
                                            class="p-1.5 rounded-lg text-gray-500 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-950/30 transition-colors"
                                            onclick={() =>
                                                openEditModal(folder)}
                                            title="Edit folder"
                                        >
                                            <Pencil size={16} />
                                        </button>
                                        <button
                                            type="button"
                                            class="p-1.5 rounded-lg text-gray-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors"
                                            onclick={() =>
                                                handleDelete(folder.id)}
                                            title="Remove folder"
                                        >
                                            <Trash2 size={16} />
                                        </button>
                                    {:else}
                                        <span class="text-xs text-gray-400"
                                            >Admin only</span
                                        >
                                    {/if}
                                </div>
                            </td>
                        </tr>
                    {/each}
                {/if}
            </tbody>
        </table>
    </div>
</div>

{#if showModal}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
    >
        <div
            class="bg-white dark:bg-gray-900 rounded-2xl w-full max-w-md shadow-2xl border border-gray-100 dark:border-gray-800 overflow-hidden"
        >
            <div
                class="flex items-center justify-between p-6 border-b border-gray-100 dark:border-gray-800"
            >
                <h3 class="text-xl font-bold text-gray-900 dark:text-white">
                    {editingFolder ? 'Edit Folder' : 'Add Music Folder'}
                </h3>
                <button
                    class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-500 transition-colors"
                    onclick={() => (showModal = false)}
                >
                    <X size={20} />
                </button>
            </div>

            <form
                class="p-6 space-y-4"
                onsubmit={(e) => {
                    e.preventDefault();
                    handleSave();
                }}
            >
                <div>
                    <label
                        for="fName"
                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                    >
                        Display Name
                    </label>
                    <input
                        id="fName"
                        type="text"
                        bind:value={folderName}
                        placeholder="e.g. My Music"
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-4 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500 outline-none transition-shadow"
                    />
                </div>
                <div>
                    <label
                        for="fPath"
                        class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
                    >
                        Absolute Path
                    </label>
                    <input
                        id="fPath"
                        type="text"
                        bind:value={folderPath}
                        placeholder="e.g. /home/user/music"
                        required
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-4 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500 outline-none transition-shadow"
                    />
                    <p class="mt-1.5 text-xs text-gray-500">
                        Ensure the application has read access to this path.
                    </p>
                </div>

                <div class="pt-4 flex gap-3">
                    <button
                        type="button"
                        class="flex-1 px-4 py-2.5 rounded-lg border border-gray-200 dark:border-gray-700 text-sm font-semibold text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                        onclick={() => (showModal = false)}
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        class="flex-1 px-4 py-2.5 rounded-lg bg-orange-600 text-white text-sm font-semibold hover:bg-orange-700 transition-colors shadow-sm shadow-orange-200 dark:shadow-none"
                    >
                        {editingFolder ? 'Save Changes' : 'Add Folder'}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}
