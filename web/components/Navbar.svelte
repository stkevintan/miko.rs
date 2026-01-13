<script lang="ts">
  import { Menu, User, RefreshCw } from 'lucide-svelte';
  import { onMount, onDestroy } from 'svelte';
  import api from '../lib/api';
  import { toast } from '../lib/toast.svelte';
  import { authStore } from '../lib/auth.svelte';
  import Dropdown from './ui/Dropdown.svelte';
  import ThemeSwitcher from './ui/ThemeSwitcher.svelte';

  let { onToggleSidebar } = $props<{
    onToggleSidebar: () => void
  }>();

  let isScanning = $state(false);
  let pollInterval: number | null = null;

  async function logout() {
    authStore.logout();
  }

  async function checkScanStatus() {
    try {
      const resp = await api.get('/scan');
      const newScanning = resp.data.scanning;
      
      if (isScanning && !newScanning) {
        toast.success('Library scan completed');
      }
      
      isScanning = newScanning;
      
      if (!isScanning && pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      } else if (isScanning && !pollInterval) {
        startPolling();
      }
    } catch (e) {
      console.error('Failed to get scan status', e);
    }
  }

  function startPolling() {
    if (pollInterval) return;
    pollInterval = setInterval(checkScanStatus, 2000);
  }

  async function triggerScan(full = false) {
    if (isScanning) return;
    
    isScanning = true;
    toast.add(full ? 'Full scan started' : 'Quick scan started', 'info');
    try {
      await api.post(`/scan?full=${full}`);
      startPolling();
    } catch (e) {
      console.error('Scan failed', e);
      isScanning = false;
      toast.error('Failed to start library scan');
    }
  }

  onMount(() => {
    checkScanStatus();
    authStore.fetchProfile();
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
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

        <!-- Scan Button Dropdown -->
        <div class="mx-3">
          <Dropdown align="auto" triggerMode="hover">
            {#snippet trigger()}
              <button 
                type="button" 
                class="p-2 text-gray-500 rounded-lg hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 {isScanning ? 'animate-spin' : ''} cursor-pointer"
                aria-label="Scan Library"
              >
                <RefreshCw size={20} />
              </button>
            {/snippet}
            {#snippet content()}
              <div class="py-1">
                <button 
                  onclick={() => triggerScan(false)}
                  class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-orange-50 hover:text-orange-600 dark:text-gray-300 dark:hover:bg-gray-600 transition cursor-pointer"
                >
                  Quick Scan
                </button>
                <button 
                  onclick={() => triggerScan(true)}
                  class="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-orange-50 hover:text-orange-600 dark:text-gray-300 dark:hover:bg-gray-600 transition cursor-pointer"
                >
                  Full Scan
                </button>
              </div>
            {/snippet}
          </Dropdown>
        </div>

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
