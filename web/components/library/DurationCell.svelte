<script lang="ts">
    let { duration }: { duration?: number } = $props();

    function formatDuration(seconds: number | undefined) {
        if (!seconds) return '--:--';
        const min = Math.floor(seconds / 60);
        const sec = Math.floor(seconds % 60);
        return `${min}:${sec.toString().padStart(2, '0')}`;
    }
</script>

<div class="text-right tabular-nums text-sm font-medium text-gray-500 dark:text-gray-400">
    {formatDuration(duration)}
</div>
