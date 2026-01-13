<script lang="ts">
  import type { Snippet } from 'svelte';
  import Navbar from './Navbar.svelte';
  import Sidebar from './Sidebar.svelte';

  let { children } = $props<{
    children?: Snippet
  }>();

  let isSidebarOpen = $state(true);

  function toggleSidebar() {
    isSidebarOpen = !isSidebarOpen;
  }
</script>

<div class="min-h-screen bg-white dark:bg-gray-900 transition-colors duration-200">
  <Navbar onToggleSidebar={toggleSidebar} />
  
  <Sidebar isOpen={isSidebarOpen} onToggle={toggleSidebar} />

  <main 
    class="pt-16 transition-all duration-300
    {isSidebarOpen ? 'lg:ml-64' : 'lg:ml-20'}"
  >
    <div class="p-4 sm:ml-0 overflow-auto">
      {#if children}
        {@render children()}
      {/if}
    </div>
  </main>
</div>
