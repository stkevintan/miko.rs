<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { onMount } from 'svelte';
    import api from '../lib/api';
    import ThemeSwitcher from '../components/ui/ThemeSwitcher.svelte';

    let username = $state('');
    let password = $state('');
    let error = $state('');
    let loading = $state(false);

    onMount(() => {
        if (localStorage.getItem('token')) {
            push('/dashboard');
        }
    });

    async function handleLogin() {
        loading = true;
        error = '';
        try {
            const response = await api.post<{ token: string }>('/login', {
                username,
                password,
            });
            localStorage.setItem('token', response.data.token);
            localStorage.setItem('username', username);
            push('/dashboard');
        } catch (e: any) {
            error = e.response?.data?.error || 'Login failed';
        } finally {
            loading = false;
        }
    }
</script>

<div
    class="min-h-screen bg-gray-100 dark:bg-gray-900 flex flex-col items-center justify-center p-4 transition-colors duration-200"
>
    <div class="fixed top-4 right-4">
        <ThemeSwitcher />
    </div>
    <div
        class="bg-white dark:bg-gray-800 p-8 rounded-2xl shadow-xl max-w-md w-full"
    >
        <h1 class="text-3xl font-bold text-orange-600 mb-6 text-center">
            Miko Login
        </h1>

        {#if error}
            <div
                class="bg-red-100 dark:bg-red-900/30 border border-red-400 dark:border-red-500/50 text-red-700 dark:text-red-400 px-4 py-3 rounded mb-4"
            >
                {error}
            </div>
        {/if}

        <form
            onsubmit={(e) => {
                e.preventDefault();
                handleLogin();
            }}
            class="space-y-4"
        >
            <div>
                <label
                    for="username"
                    class="block text-sm font-medium text-gray-700 dark:text-gray-300"
                    >Username</label
                >
                <input
                    type="text"
                    id="username"
                    bind:value={username}
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500 dark:bg-gray-700 dark:text-white"
                    required
                />
            </div>
            <div>
                <label
                    for="password"
                    class="block text-sm font-medium text-gray-700 dark:text-gray-300"
                    >Password</label
                >
                <input
                    type="password"
                    id="password"
                    bind:value={password}
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-orange-500 focus:border-orange-500 dark:bg-gray-700 dark:text-white"
                    required
                />
            </div>
            <button
                type="submit"
                disabled={loading}
                class="w-full bg-orange-600 hover:bg-orange-700 text-white font-semibold py-2 px-6 rounded-lg transition duration-200 disabled:opacity-50"
            >
                {loading ? 'Logging in...' : 'Login'}
            </button>
        </form>
    </div>
</div>
