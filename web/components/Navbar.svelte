<script lang="ts">
  import { Menu, Search, User } from 'lucide-svelte';
  import { onMount } from 'svelte';
  import { authStore } from '../lib/auth.svelte';
  import { submitLibrarySearch } from '../lib/librarySearch';
  import { isActive, navigate } from '../router';
  import Dropdown from './ui/Dropdown.svelte';
  import ThemeSwitcher from './ui/ThemeSwitcher.svelte';
  import ScanButton from './ScanButton.svelte';
  import LibrarySearchForm from './LibrarySearchForm.svelte';

  let { onToggleSidebar } = $props<{
    onToggleSidebar: () => void
  }>();

  let isSearchOpen = $state(false);

  async function logout() {
    authStore.logout();
  }

  onMount(() => {
    authStore.fetchProfile();
  });

  function handleLibrarySearch(event: Event) {
    event.preventDefault();
    if (!isActive('/library')) {
      navigate('/library');
    }
    submitLibrarySearch();
    isSearchOpen = false;
  }
</script>

<nav class="fixed top-0 z-40 w-full bg-white border-b border-gray-200 dark:bg-gray-800 dark:border-gray-700">
  <div class="px-3 py-3 lg:px-5 lg:pl-3">
    <div class="flex items-center gap-4">
      <div class="flex items-center justify-start rtl:justify-end flex-1 min-w-0">
        <button 
          onclick={onToggleSidebar}
          class="inline-flex items-center p-2 text-sm text-gray-500 rounded-lg hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-gray-200 dark:text-gray-400 dark:hover:bg-gray-700 dark:focus:ring-gray-600 cursor-pointer"
        >
          <Menu size={24} />
        </button>
        <a href="/" class="flex ms-2 md:me-24">
          <span class="self-center text-xl font-semibold sm:text-2xl whitespace-nowrap dark:text-white">Miko</span>
        </a>
      </div>

      <div class="flex-1 flex items-center justify-center min-w-0">
        <LibrarySearchForm
          onSubmit={handleLibrarySearch}
          className="relative w-full max-w-md hidden md:block"
        />
      </div>
      
      <div class="flex items-center justify-end flex-1 min-w-0">
        <button
          type="button"
          class="md:hidden p-2 rounded-lg text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
          onclick={() => (isSearchOpen = true)}
          aria-label="Open search"
        >
          <Search size={20} />
        </button>
        <!-- Scan Button -->
        <div class="mx-1">
          <ScanButton />
        </div>

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
                <div class="flex items-center gap-2">
                  <p class="text-sm text-gray-900 dark:text-white truncate" role="none">
                    {authStore.user?.username || 'Loading...'}
                  </p>
                  {#if authStore.user?.adminRole}
                    <span
                      class="px-1 py-0.5 rounded bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-400 text-[9px] font-bold uppercase tracking-tight"
                    >
                      Admin
                    </span>
                  {/if}
                </div>
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

  {#if isSearchOpen}
    <div class="fixed inset-0 z-50 md:hidden">
      <button
        class="absolute inset-0 bg-black/40"
        onclick={() => (isSearchOpen = false)}
        aria-label="Close search"
      ></button>
      <div class="absolute top-16 left-4 right-4 bg-white dark:bg-gray-900 rounded-xl p-4 shadow-lg border border-gray-100 dark:border-gray-800">
        <LibrarySearchForm
          onSubmit={handleLibrarySearch}
          className="relative w-full"
        />
      </div>
    </div>
  {/if}
</nav>
