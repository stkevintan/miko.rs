<script lang="ts">
  import { link, location } from 'svelte-spa-router';
  import { Home, Library, Music, Settings, User, X } from 'lucide-svelte';

  let { isOpen = true, onToggle } = $props<{
    isOpen?: boolean,
    onToggle?: () => void
  }>();

  const navItems = [
    { name: 'Home', path: '/dashboard', icon: Home },
    { name: 'Library', path: '/library', icon: Library },
    { name: 'Artists', path: '/artists', icon: User },
    { name: 'Albums', path: '/albums', icon: Music },
    { name: 'Settings', path: '/settings', icon: Settings },
  ];
</script>

<!-- Mobile Overlay -->
{#if isOpen}
  <button 
    class="fixed inset-0 bg-black/50 z-20 transition-opacity lg:hidden w-full h-full cursor-default"
    onclick={onToggle}
    aria-label="Close sidebar"
  ></button>
{/if}

<aside 
  class="fixed top-0 left-0 z-30 h-screen transition-transform bg-white border-r border-gray-200 dark:bg-gray-800 dark:border-gray-700
  {isOpen ? 'w-64 translate-x-0' : 'w-0 -translate-x-full lg:w-20 lg:translate-x-0'} overflow-hidden"
>
  <div class="flex flex-col h-full">
    <div class="flex items-center justify-between h-16 px-4 border-b">
      <span class="text-xl font-bold text-orange-600 {isOpen ? 'block' : 'hidden lg:hidden'}">Miko</span>
      <button 
        class="p-2 rounded-lg hover:bg-gray-100 lg:hidden"
        onclick={onToggle}
      >
        <X size={20} />
      </button>
    </div>

    <nav class="flex-1 px-3 py-4 space-y-1">
      {#each navItems as item}
        <a 
          href={item.path}
          use:link
          class="flex items-center p-2 text-gray-900 rounded-lg dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group
          {$location === item.path ? 'bg-orange-50 text-orange-600 dark:bg-gray-700' : ''}"
        >
          <item.icon size={20} class={$location === item.path ? 'text-orange-600' : 'text-gray-500'} />
          <span class="ms-3 transition-opacity duration-300 {isOpen ? 'opacity-100' : 'opacity-0 lg:hidden'}">{item.name}</span>
        </a>
      {/each}
    </nav>
  </div>
</aside>
