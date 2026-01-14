<script lang="ts">
    import { Music } from "lucide-svelte";

    interface Props {
        name: string;
        username?: string;
        iconSrc?: string;
        iconClass?: string;
        statusColor?: string;
        connected?: boolean;
    }

    let {
        name,
        username,
        iconSrc,
        iconClass = '',
        statusColor = 'text-gray-400',
        connected = false,
    }: Props = $props();

    let title = $derived(connected ? `${name}: ${username}` : `${name}: Disconnected`);
</script>

<div
    class="flex items-center gap-2.5 group cursor-pointer {connected ? '' : 'opacity-40 grayscale hover:opacity-100 hover:grayscale-0'} transition-all"
    {title}
>
    <div class="relative">
        <div
            class="w-9 h-9 rounded-xl flex items-center justify-center overflow-hidden shadow-sm transition-transform group-hover:scale-110 {iconClass}"
        >
            {#if iconSrc}
                <img
                    src={iconSrc}
                    alt={name}
                    class="w-full h-full object-contain"
                />
            {:else}
                <Music size={18} />
            {/if}
        </div>
        {#if connected}
            <span
                class="absolute -top-0.5 -right-0.5 block h-2.5 w-2.5 rounded-full bg-green-500 border-2 border-white dark:border-gray-800"
            ></span>
        {/if}
    </div>
    <div class="min-w-0 hidden @[500px]:block">
        <p
            class="text-[11px] font-bold leading-tight truncate {connected ? 'text-gray-900 dark:text-white' : 'text-gray-400 dark:text-gray-500'}"
        >
            {connected ? username : 'Sign in'}
        </p>
        <p
            class="text-[9px] {statusColor} font-medium uppercase tracking-tighter leading-none mt-0.5"
        >
            {name}
        </p>
    </div>
</div>
