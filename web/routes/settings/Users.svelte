<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import { authStore } from '../../lib/auth.svelte';
    import type { SubsonicResponse, SubsonicUser } from '../../lib/types';

    let users = $state<SubsonicUser[]>([]);
    let loading = $state(false);
    let showDialog = $state<'create' | 'edit' | null>(null);
    let dialogUser = $state({
        username: '',
        password: '',
        email: '',
        adminRole: false
    });

    async function fetchUsers() {
        if (!authStore.user?.adminRole) return;
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getUsers');
            users = response.data.users?.user ?? [];
        } catch (error) {
            console.error('Failed to fetch users:', error);
            toast.error('Failed to load users');
        } finally {
            loading = false;
        }
    }

    function openCreate() {
        dialogUser = { username: '', password: '', email: '', adminRole: false };
        showDialog = 'create';
    }

    function openEdit(user: SubsonicUser) {
        dialogUser = {
            username: user.username,
            password: '',
            email: user.email || '',
            adminRole: user.adminRole
        };
        showDialog = 'edit';
    }

    async function saveUser() {
        if (!dialogUser.username) {
            toast.error('Username is required');
            return;
        }
        if (showDialog === 'create' && !dialogUser.password) {
            toast.error('Password is required');
            return;
        }

        loading = true;
        try {
            const endpoint = showDialog === 'create' ? '/createUser' : '/updateUser';
            await api.get(endpoint, {
                params: {
                    username: dialogUser.username,
                    password: dialogUser.password || undefined,
                    email: dialogUser.email || undefined,
                    adminRole: dialogUser.adminRole
                }
            });
            toast.success(`User ${showDialog === 'create' ? 'created' : 'updated'} successfully`);
            showDialog = null;
            await fetchUsers();
        } catch (error: any) {
            toast.error(error.message || `Failed to ${showDialog} user`);
        } finally {
            loading = false;
        }
    }

    async function deleteUser(username: string) {
        if (username === authStore.user?.username) {
            toast.error('You cannot delete yourself');
            return;
        }
        if (!confirm(`Are you sure you want to delete user "${username}"?`)) return;

        loading = true;
        try {
            await api.get('/deleteUser', { params: { username } });
            toast.success('User deleted successfully');
            await fetchUsers();
        } catch (error: any) {
            toast.error(error.message || 'Failed to delete user');
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        authStore.fetchProfile();
        fetchUsers();
    });
</script>

<div class="flex items-center mb-4 gap-6">
    <h2 class="mr-auto text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Users
    </h2>
    {#if authStore.user?.adminRole}
        <button
            type="button"
            class="px-3 py-2 rounded-lg bg-orange-600 text-white text-sm font-semibold hover:bg-orange-700"
            onclick={openCreate}
        >
            Create User
        </button>
    {/if}
</div>

{#if showDialog}
    <div class="mb-6 bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm p-6">
        <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
            {showDialog === 'create' ? 'Create New User' : `Edit User: ${dialogUser.username}`}
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
                <label for="username" class="block text-xs font-medium text-gray-500 uppercase mb-1">Username</label>
                <input
                    id="username"
                    type="text"
                    bind:value={dialogUser.username}
                    disabled={showDialog === 'edit'}
                    class="w-full px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-transparent text-sm focus:ring-2 focus:ring-orange-500 outline-none disabled:opacity-50"
                />
            </div>
            <div>
                <label for="email" class="block text-xs font-medium text-gray-500 uppercase mb-1">Email</label>
                <input
                    id="email"
                    type="email"
                    bind:value={dialogUser.email}
                    class="w-full px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-transparent text-sm focus:ring-2 focus:ring-orange-500 outline-none"
                    placeholder="email@example.com"
                />
            </div>
            <div>
                <label for="password" class="block text-xs font-medium text-gray-500 uppercase mb-1">
                    {showDialog === 'create' ? 'Password' : 'New Password (leave blank to keep current)'}
                </label>
                <input
                    id="password"
                    type="password"
                    bind:value={dialogUser.password}
                    class="w-full px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-transparent text-sm focus:ring-2 focus:ring-orange-500 outline-none"
                    placeholder={showDialog === 'create' ? '' : '••••••••'}
                />
            </div>
            <div class="flex items-center gap-2 mt-5">
                <input
                    id="adminRole"
                    type="checkbox"
                    bind:checked={dialogUser.adminRole}
                    class="rounded border-gray-300 text-orange-600 focus:ring-orange-500"
                />
                <label for="adminRole" class="text-sm font-medium text-gray-700 dark:text-gray-300">Admin Role</label>
            </div>
        </div>
        <div class="flex justify-end gap-3 mt-6">
            <button
                type="button"
                class="px-4 py-2 rounded-lg border border-gray-200 dark:border-gray-700 text-sm font-semibold hover:bg-gray-50 dark:hover:bg-gray-800"
                onclick={() => showDialog = null}
            >
                Cancel
            </button>
            <button
                type="button"
                class="px-4 py-2 rounded-lg bg-orange-600 text-white text-sm font-semibold hover:bg-orange-700 disabled:opacity-50"
                disabled={loading}
                onclick={saveUser}
            >
                {loading ? 'Saving...' : 'Save User'}
            </button>
        </div>
    </div>
{/if}

{#if !authStore.user?.adminRole}
    <div class="bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm p-6">
        <p class="text-sm text-gray-500 dark:text-gray-400">
            Admin access is required to manage users.
        </p>
    </div>
{:else}
    <div class="bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm">
        <div class="overflow-x-auto">
            <table class="min-w-full text-sm">
                <thead class="text-left text-gray-500 dark:text-gray-400">
                    <tr>
                        <th class="px-4 py-3">Username</th>
                        <th class="px-4 py-3">Email</th>
                        <th class="px-4 py-3 text-right">Admin</th>
                        <th class="px-4 py-3 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {#if loading && users.length === 0}
                        <tr>
                            <td class="px-4 py-6 text-center text-gray-500 dark:text-gray-400" colspan="4">
                                Loading users...
                            </td>
                        </tr>
                    {:else if users.length === 0}
                        <tr>
                            <td class="px-4 py-6 text-center text-gray-500 dark:text-gray-400" colspan="4">
                                No users found.
                            </td>
                        </tr>
                    {:else}
                        {#each users as user}
                            <tr class="border-t border-gray-100 dark:border-gray-800">
                                <td class="px-4 py-3 font-medium text-gray-900 dark:text-white">
                                    {user.username}
                                </td>
                                <td class="px-4 py-3 text-gray-500 dark:text-gray-400">
                                    {user.email || '—'}
                                </td>
                                <td class="px-4 py-3 text-right text-gray-500 dark:text-gray-400">
                                    {user.adminRole ? 'Yes' : 'No'}
                                </td>
                                <td class="px-4 py-3 text-right">
                                    <div class="flex justify-end gap-2">
                                        <button
                                            type="button"
                                            class="px-3 py-1.5 rounded-lg text-xs font-semibold border border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800"
                                            onclick={() => openEdit(user)}
                                        >
                                            Edit
                                        </button>
                                        <button
                                            type="button"
                                            class="px-3 py-1.5 rounded-lg text-xs font-semibold border border-red-200 dark:border-red-900/30 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 disabled:opacity-50"
                                            disabled={user.username === authStore.user?.username}
                                            onclick={() => deleteUser(user.username)}
                                        >
                                            Delete
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    {/if}
                </tbody>
            </table>
        </div>
    </div>
{/if}
