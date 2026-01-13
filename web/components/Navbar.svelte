<script lang="ts">
  import { Menu, User } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { authStore } from '../lib/auth.svelte';
  import Dropdown from './ui/Dropdown.svelte';
  import ThemeSwitcher from './ui/ThemeSwitcher.svelte';

  let { onToggleSidebar } = $props<{
    onToggleSidebar: () => void
  }>();

  async function logout() {
    authStore.logout();
  }

  onMount(() => {
    authStore.fetchProfile();
  });
</script>

<nav class="fixed top-0 z-40 w-full bg-white border-b border-gray-200 dark:bg-gray-800 dark:border-gray-700">
  <div class="px-3 py-3 lg:px-5 lg:pl-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center justify-start rtl:justify-end">
        <button 
          onclick={onToggleSidebar}
          class="inline-flex items-center p-2 text-sm text-gray-500 rounded-lg hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600 cursor-pointer"
        >
          <Menu size={24} />
        </button>
        <a href="/" class="flex ms-2 md:me-24">
          <span class="self-center text-xl font-semibold sm:text-2xl whitespace-nowrap dark:text-white">Music</span>
        </a>
      </div>
      
      <div class="flex items-center">
        <!-- Theme Switcher -->
        <ThemeSwitcher class="mx-1" />

        <div class="ms-3">
          <Dropdown align="auto">
            {#snippet trigger()}
              <button 
                type="button" 
                class="flex items-center text-sm bg-gray-800 rounded-full focus:ring-4 focus:ring-gray-300 dark:focus:ring-gray-600 cursor-pointer"
              >
                <span class="sr-only">Open user menu</span>
                <div class="w-8 h-8 rounded-full bg-orange-500 flex items-center justify-center text-white font-medium">
                  {#if authStore.user}
                    {authStore.user.username[0].toUpperCase()}
                  {:else}
                    <User size={20} />
                  {/if}
                </div>
              </button>
            {/snippet}
            {#snippet content()}
              <div class="px-4 py-3 border-b dark:border-gray-600">
                <p class="text-sm text-gray-900 dark:text-white truncate" role="none">
                  {authStore.user?.username || 'Loading...'}
                </p>
                {#if authStore.user?.email}
                  <p class="text-sm font-medium text-gray-500 truncate dark:text-gray-400" role="none">
                    {authStore.user.email}
                  </p>
                {/if}
              </div>
              <ul class="py-1" role="none">
                <li>
                  <a href="/dashboard" class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600" role="menuitem">Dashboard</a>
                </li>
                <li>
                  <a href="/settings" class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600" role="menuitem">Settings</a>
                </li>
                <li>
                  <button 
                    onclick={logout}
                    class="w-full text-left block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600 cursor-pointer" 
                    role="menuitem"
                  >
                    Logout
                  </button>
                </li>
              </ul>
            {/snippet}
          </Dropdown>
        </div>
      </div>
    </div>
  </div>
</nav>
