<script lang="ts">
    import { onMount } from 'svelte';
    import { authStore } from '../../lib/auth.svelte';
    import { toast } from '../../lib/toast.svelte';
    import { api } from '../../lib/api';

    let email = $state('');
    let currentPassword = $state('');
    let newPassword = $state('');
    let confirmPassword = $state('');

    onMount(async () => {
        await authStore.fetchProfile();
        if (authStore.user) {
            email = authStore.user.email || '';
        }
    });

    async function handleSave() {
        if (newPassword && newPassword !== confirmPassword) {
            toast.error('New passwords do not match.');
            return;
        }

        if (!currentPassword) {
            toast.error('Please enter your current password to save changes.');
            return;
        }
        
        try {
            await api.post('/profile', {
                email,
                current_password: currentPassword,
                new_password: newPassword || undefined
            });
            toast.add('Profile updated successfully', 'success');
            
            // Refresh local user data
            if (authStore.user) {
                authStore.user.email = email;
            }

            // Clear password fields
            currentPassword = '';
            newPassword = '';
            confirmPassword = '';
        } catch (e: any) {
            toast.error(e.response?.data?.error || 'Failed to update profile');
        }
    }
</script>

<div class="flex items-center mb-4 gap-6">
    <h2 class="mr-auto text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Profile
    </h2>
</div>

<div class="max-w-4xl">
    <form
        class="bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 p-6"
        onsubmit={(e) => {
            e.preventDefault();
            handleSave();
        }}
    >
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-6">Account Settings</h3>
        
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
            <div class="space-y-4">
                <h4 class="text-sm font-medium text-gray-400 uppercase tracking-wider">General</h4>
                <div>
                    <label for="username" class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-2">
                        Username
                    </label>
                    <input
                        id="username"
                        type="text"
                        value={authStore.user?.username || ''}
                        disabled
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 px-3 py-2 text-sm text-gray-500 dark:text-gray-400 cursor-not-allowed"
                    />
                </div>
                <div>
                    <label for="email" class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-2">
                        Email Address
                    </label>
                    <input
                        id="email"
                        type="email"
                        bind:value={email}
                        placeholder="name@example.com"
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500"
                    />
                </div>
            </div>

            <div class="space-y-4">
                <h4 class="text-sm font-medium text-gray-400 uppercase tracking-wider">Security</h4>
                <div>
                    <label for="newPassword" class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-2">
                        New password (leave blank to keep current)
                    </label>
                    <input
                        id="newPassword"
                        type="password"
                        bind:value={newPassword}
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500"
                    />
                </div>
                <div>
                    <label for="confirmPassword" class="block text-sm font-medium text-gray-600 dark:text-gray-300 mb-2">
                        Confirm new password
                    </label>
                    <input
                        id="confirmPassword"
                        type="password"
                        bind:value={confirmPassword}
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500"
                    />
                </div>
            </div>
        </div>

        <div class="pt-6 border-t border-gray-100 dark:border-gray-800">
            <div class="max-w-xs space-y-4">
                <div>
                    <label for="currentPassword" class="block text-sm font-medium text-gray-900 dark:text-white mb-2">
                        Confirm changes with current password
                    </label>
                    <input
                        id="currentPassword"
                        type="password"
                        bind:value={currentPassword}
                        required
                        class="w-full rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-white focus:ring-2 focus:ring-orange-500"
                    />
                </div>
                <button
                    type="submit"
                    class="w-full px-4 py-2 rounded-lg bg-orange-600 text-white text-sm font-semibold hover:bg-orange-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    disabled={!currentPassword}
                >
                    Save Changes
                </button>
            </div>
        </div>
    </form>
</div>
