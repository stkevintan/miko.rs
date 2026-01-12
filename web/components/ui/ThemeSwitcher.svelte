<script lang="ts">
  import { Sun, Moon, Monitor } from 'lucide-svelte';
  import { themeManager } from '../../lib/theme.svelte';
  import Dropdown from './Dropdown.svelte';

  let { class: className = '' } = $props<{ class?: string }>();
</script>

<div class={className}>
  <Dropdown align="auto" triggerMode="hover">
    {#snippet trigger()}
      <button 
        class="p-2 text-gray-500 rounded-lg hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700 cursor-pointer"
        aria-label="Toggle Theme"
      >
        {#if themeManager.theme === 'dark'}
          <Moon size={20} />
        {:else if themeManager.theme === 'light'}
          <Sun size={20} />
        {:else}
          <Monitor size={20} />
        {/if}
      </button>
    {/snippet}
    
    {#snippet content()}
      <div class="py-1">
        <button 
          onclick={() => themeManager.setTheme('light')}
          class="w-full flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600 {themeManager.theme === 'light' ? 'bg-orange-50 text-orange-600 dark:bg-gray-800' : ''} cursor-pointer"
        >
          <Sun size={16} class="mr-2" />
          Light
        </button>
        <button 
          onclick={() => themeManager.setTheme('dark')}
          class="w-full flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600 {themeManager.theme === 'dark' ? 'bg-orange-50 text-orange-600 dark:bg-gray-800' : ''} cursor-pointer"
        >
          <Moon size={16} class="mr-2" />
          Dark
        </button>
        <button 
          onclick={() => themeManager.setTheme('system')}
          class="w-full flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-600 {themeManager.theme === 'system' ? 'bg-orange-50 text-orange-600 dark:bg-gray-800' : ''} cursor-pointer"
        >
          <Monitor size={16} class="mr-2" />
          System
        </button>
      </div>
    {/snippet}
  </Dropdown>
</div>
