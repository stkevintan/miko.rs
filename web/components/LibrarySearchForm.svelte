<script lang="ts">
  import { Search } from 'lucide-svelte';
  import { librarySearchQuery } from '../lib/librarySearch';

  export let className = '';
  export let onSubmit: (event: Event) => void = (event) => event.preventDefault();
</script>

<form onsubmit={onSubmit} class={className}>
  <input
    type="text"
    bind:value={$librarySearchQuery}
    placeholder="Search songs, artists, albums..."
    class="w-full pl-10 pr-4 py-2 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-orange-500 outline-none transition-all dark:text-white"
  />
  <Search class="absolute left-3 top-2.5 text-gray-400" size={20} />
  <button type="submit" class="hidden">Search</button>
</form>
