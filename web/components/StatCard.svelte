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
        icon: Icon 
    } = $props<{
        label: string;
        value: string | number;
        color?: keyof typeof colorClasses;
        icon: any;
    }>();
</script>

<div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700 flex items-center">
    <div class="p-3 rounded-xl mr-4 {colorClasses[color as keyof typeof colorClasses]}">
        <Icon size={24} />
    </div>
    <div>
        <p class="text-sm text-gray-500 dark:text-gray-400">{label}</p>
        <p class="text-2xl font-bold text-gray-900 dark:text-white">{value}</p>
    </div>
</div>
