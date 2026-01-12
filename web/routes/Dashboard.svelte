<script lang="ts">
    import { push } from 'svelte-spa-router';
    import { onMount } from 'svelte';

    let token = $state(localStorage.getItem('token'));

    onMount(() => {
        if (!token) {
            push('/login');
        }
    });

    function handleLogout() {
        localStorage.removeItem('token');
        push('/login');
    }
</script>

<div class="min-h-screen bg-gray-50 p-8">
    <header class="flex justify-between items-center mb-8">
        <h1 class="text-3xl font-bold text-gray-900">Miko Dashboard</h1>
        <button
            onclick={handleLogout}
            class="text-gray-600 hover:text-orange-600 font-medium transition"
        >
            Logout
        </button>
    </header>

    <main class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-semibold mb-2">Welcome!</h2>
            <p class="text-gray-600">
                You are successfully logged in to your Miko Subsonic server.
            </p>
        </div>

        <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-semibold mb-2">Music Library</h2>
            <p class="text-gray-600">
                Your music library is currently being scanned and updated.
            </p>
        </div>

        <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-xl font-semibold mb-2">API Access</h2>
            <p class="text-gray-600">Authenticated via JWT Token.</p>
        </div>
    </main>
</div>
