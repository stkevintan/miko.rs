<script lang="ts">
    import { authStore } from '../lib/auth.svelte';
    import DashboardCard from './DashboardCard.svelte';
    import ConnectionItem from './ConnectionItem.svelte';
    import { User } from 'lucide-svelte';
    import spotifyIcon from '../icons/spotify.svg';
    import lastfmIcon from '../icons/lastfm.svg';
    import neteaseIcon from '../icons/netease.svg';
    import qqmusicIcon from '../icons/qqmusic.svg';
</script>

{#if authStore.loading && !authStore.user}
    <div
        class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 dark:bg-gray-800 dark:border-gray-700 animate-pulse h-full"
    >
        <div class="h-6 w-32 bg-gray-200 dark:bg-gray-700 rounded mb-6"></div>
        <div class="flex items-start">
            <div
                class="w-16 h-16 rounded-2xl bg-gray-200 dark:bg-gray-700 mr-4 shrink-0"
            ></div>
            <div class="flex-1 space-y-3">
                <div class="h-5 w-24 bg-gray-200 dark:bg-gray-700 rounded"></div>
                <div class="h-4 w-32 bg-gray-200 dark:bg-gray-700 rounded"></div>
            </div>
        </div>
    </div>
{:else if authStore.user}
    <DashboardCard
        title="Account & Connections"
        icon={User}
        iconClass="text-green-600"
        class="h-full flex flex-col"
    >
        <div
            class="flex-1 flex flex-row flex-wrap items-center justify-start w-full gap-8"
        >
            <div class="flex items-center min-w-[280px] overflow-hidden mr-auto">
                <div
                    class="w-20 h-20 rounded-3xl bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center text-white text-3xl font-bold mr-5 shrink-0 shadow-lg"
                >
                    {authStore.user.username[0].toUpperCase()}
                </div>
                <div class="min-w-0">
                    <div class="flex items-center gap-2">
                        <p
                            class="text-xl font-bold text-gray-900 dark:text-white truncate"
                        >
                            {authStore.user.username}
                        </p>
                        {#if authStore.user.adminRole}
                            <span
                                class="px-1.5 py-0.5 rounded-md bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 text-[10px] font-bold uppercase tracking-wider"
                            >
                                Admin
                            </span>
                        {/if}
                    </div>
                    <p
                        class="text-sm text-gray-500 dark:text-gray-400 truncate mt-1"
                    >
                        {authStore.user.email || 'No email provided'}
                    </p>
                </div>
            </div>
            <div
                class="flex flex-wrap justify-start gap-x-6 gap-y-3 @[620px]:grid @[620px]:grid-cols-2 @[620px]:max-w-[400px]"
            >
                <ConnectionItem
                    name="Netease"
                    username="小星星OvO"
                    iconSrc={neteaseIcon}
                    statusColor="text-red-600"
                    connected={true}
                />

                <ConnectionItem
                    name="QQ Music"
                    iconSrc={qqmusicIcon}
                    statusColor="text-yellow-600"
                    connected={false}
                />

                <ConnectionItem
                    name="Spotify"
                    username="stkevintan"
                    iconSrc={spotifyIcon}
                    statusColor="text-green-600"
                    connected={true}
                />

                <ConnectionItem
                    name="Last.fm"
                    iconSrc={lastfmIcon}
                    statusColor="text-gray-400"
                    connected={false}
                />
            </div>
        </div>
    </DashboardCard>
{/if}
