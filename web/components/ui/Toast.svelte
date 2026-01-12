<script lang="ts">
  import { CheckCircle, XCircle, Info, X } from 'lucide-svelte';
  import { flip } from 'svelte/animate';
  import { fade, fly } from 'svelte/transition';
  import { toast } from '../../lib/toast.svelte';
</script>

<div class="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
  {#each toast.toasts as item (item.id)}
    <div 
      animate:flip={{ duration: 300 }}
      in:fly={{ y: 20, duration: 300 }}
      out:fade={{ duration: 200 }}
      class="pointer-events-auto flex items-center w-full max-w-xs p-4 text-gray-500 bg-white rounded-lg shadow-lg dark:text-gray-400 dark:bg-gray-800 border dark:border-gray-700"
      role="alert"
    >
      <div class="inline-flex items-center justify-center shrink-0 w-8 h-8 rounded-lg 
        {item.type === 'success' ? 'text-green-500 bg-green-100 dark:bg-green-800 dark:text-green-200' : ''}
        {item.type === 'error' ? 'text-red-500 bg-red-100 dark:bg-red-800 dark:text-red-200' : ''}
        {item.type === 'info' ? 'text-blue-500 bg-blue-100 dark:bg-blue-800 dark:text-blue-200' : ''}"
      >
        {#if item.type === 'success'}
          <CheckCircle size={20} />
        {:else if item.type === 'error'}
          <XCircle size={20} />
        {:else}
          <Info size={20} />
        {/if}
      </div>
      <div class="ms-3 text-sm font-normal">{item.message}</div>
      <button 
        type="button" 
        onclick={() => toast.remove(item.id)}
        class="ms-auto -mx-1.5 -my-1.5 bg-white text-gray-400 hover:text-gray-900 rounded-lg focus:ring-2 focus:ring-gray-300 p-1.5 hover:bg-gray-100 inline-flex items-center justify-center h-8 w-8 dark:text-gray-500 dark:hover:text-white dark:bg-gray-800 dark:hover:bg-gray-700 cursor-pointer" 
        aria-label="Close"
      >
        <X size={16} />
      </button>
    </div>
  {/each}
</div>
