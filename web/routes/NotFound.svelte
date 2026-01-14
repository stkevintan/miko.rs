<script lang="ts">
  import { navigate } from '../router';
  import { FileQuestion, ArrowLeft } from 'lucide-svelte';
</script>

<div class="min-h-[70vh] flex flex-col items-center justify-center p-4 text-center">
  <div class="relative mb-6">
    <div class="absolute inset-0 bg-orange-500/10 blur-3xl rounded-full"></div>
    <FileQuestion size={120} class="text-orange-500 relative animate-pulse" />
  </div>
  
  <h1 class="text-6xl font-black text-gray-900 dark:text-white mb-4 tracking-tighter">
    404
  </h1>
  
  <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-200 mb-2">
    Page Not Found
  </h2>
  
  <p class="text-gray-500 dark:text-gray-400 max-w-md mb-8">
    The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.
  </p>
  
  <button 
    onclick={() => navigate('/')}
    class="inline-flex items-center gap-2 px-6 py-3 bg-orange-500 hover:bg-orange-600 text-white font-semibold rounded-xl transition-all shadow-lg shadow-orange-500/25 active:scale-95 cursor-pointer"
  >
    <ArrowLeft size={20} />
    Back to Dashboard
  </button>
</div>

