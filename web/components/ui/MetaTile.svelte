<script lang="ts">
    let { label, value }: { label: string; value: string | number | undefined } = $props();
</script>

<div class="p-3 bg-gray-50 dark:bg-gray-900/40 rounded-xl border border-gray-100 dark:border-gray-700/50">
    <div class="text-[10px] font-bold text-gray-500 uppercase mb-1">{label}</div>
    <div class="text-sm font-bold truncate">{value ?? '-'}</div>
</div>
