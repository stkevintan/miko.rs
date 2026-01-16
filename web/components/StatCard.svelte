<script lang="ts">
    const colorClasses = {
        orange: 'bg-orange-100 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400',
        blue: 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400',
        green: 'bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400',
        purple: 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400',
        red: 'bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400',
    } as const;

    let { 
        label, 
        value, 
        color = 'blue', 
        icon: Icon,
        children,
        onclick
    } = $props<{
        label: string;
        value: string | number | undefined;
        color?: keyof typeof colorClasses;
        icon: any;
        children?: import('svelte').Snippet;
        onclick?: () => void;
    }>();
</script>

<button 
    type="button"
    class="w-full text-left bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700 flex items-center relative transition-all duration-200 group {onclick ? 'hover:shadow-md hover:border-orange-200 dark:hover:border-orange-900/50' : ''}"
    {onclick}
    disabled={!onclick}
>
    <div class="p-3 rounded-xl mr-4 {colorClasses[color as keyof typeof colorClasses]} group-hover:scale-110 transition-transform">
        <Icon size={24} />
    </div>
    <div class="flex-1 min-w-0">
        <p class="text-sm text-gray-500 dark:text-gray-400 font-medium">{label}</p>
        <p class="text-2xl font-bold text-gray-900 dark:text-white mt-0.5">{value || 0}</p>
    </div>
    {#if children}
        <div class="absolute top-3 right-3">
            {@render children()}
        </div>
    {/if}
</button>
