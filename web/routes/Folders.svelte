<script lang="ts">
    import { onMount } from 'svelte';
    import { navigate } from '../router';
    import { api } from '../lib/api';
    import { toast } from '../lib/toast.svelte';
    import type { Song, SubsonicResponse, MusicFolder, Directory } from '../lib/types';
    import { Folder, ChevronLeft } from 'lucide-svelte';
    import DataTable from '../components/ui/DataTable.svelte';
    import DurationCell from '../components/library/DurationCell.svelte';
    import TitleCell from '../components/library/TitleCell.svelte';
    import SongMetadataDrawer from '../components/library/SongMetadataDrawer.svelte';
    import { setLibraryViewKey } from '../lib/libraryView';

    let currentDirectory = $state<Directory | null>(null);
    let musicFolders = $state<MusicFolder[]>([]);
    let loading = $state(true);
    let history = $state<string[]>([]);
    let selectedSongId = $state<string | null>(null);
    let isDrawerOpen = $state(false);
    let selectedFolderId = $state<string>('');

    async function fetchMusicFolders() {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getMusicFolders');
            musicFolders = response.data.musicFolders?.musicFolder || [];
            
            if (musicFolders.length > 0 && !currentDirectory) {
                const firstFolder = musicFolders[0];
                selectedFolderId = firstFolder.directoryId || firstFolder.id.toString();
                fetchDirectory(selectedFolderId);
            }
        } catch (error) {
            console.error('Failed to fetch music folders:', error);
            toast.error('Failed to load music folders');
        } finally {
            loading = false;
        }
    }

    async function fetchDirectory(id: string) {
        loading = true;
        try {
            const response = await api.get<SubsonicResponse>('/getMusicDirectory', {
                params: { id }
            });
            if (response.data.directory) {
                currentDirectory = response.data.directory;
            }
        } catch (error) {
            console.error('Failed to fetch directory:', error);
            toast.error('Failed to load directory');
        } finally {
            loading = false;
        }
    }

    function navigateTo(id: string) {
        if (currentDirectory) {
            history = [...history, currentDirectory.id];
        }
        fetchDirectory(id);
    }

    function goBack() {
        if (history.length === 0) return;
        const previous = history[history.length - 1];
        history = history.slice(0, -1);
        fetchDirectory(previous);
    }

    function handleRowClick(item: Song) {
        if (item.isDir) return;
        selectedSongId = item.id;
        isDrawerOpen = true;
    }

    function getRelativePath(itemPath?: string) {
        if (!itemPath || !currentDirectory?.path) return itemPath || '';
        
        // Ensure directory path ends with a slash for clean replacement
        const dirPath = currentDirectory.path.endsWith('/') 
            ? currentDirectory.path 
            : currentDirectory.path + '/';
            
        if (itemPath.startsWith(dirPath)) {
            return itemPath.slice(dirPath.length);
        }
        
        // Fallback: if it starts with the dirPath without the trailing slash
        if (itemPath.startsWith(currentDirectory.path)) {
            const relative = itemPath.slice(currentDirectory.path.length);
            return relative.startsWith('/') ? relative.slice(1) : relative;
        }

        return itemPath;
    }

    onMount(() => {
        setLibraryViewKey('folders');
        fetchMusicFolders();
    });

</script>

{#snippet header()}
    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Path</th>
    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Artist</th>
    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Album</th>
    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider w-24">Duration</th>
{/snippet}

{#snippet row(item: Song)}
    <td class="px-4 py-3">
        {#if item.isDir}
            <button 
                onclick={(e) => {
                    e.stopPropagation();
                    navigateTo(item.id);
                }}
                class="flex items-center gap-3 py-1 min-w-0 overflow-hidden group/link"
            >
                <div class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-800 flex items-center justify-center text-gray-500 group-hover/link:bg-orange-100 dark:group-hover/link:bg-orange-900/30 group-hover/link:text-orange-600 transition-colors">
                    <Folder size={20} />
                </div>
                <div class="text-sm font-semibold text-gray-900 dark:text-white group-hover/link:text-orange-600 truncate transition-colors">
                    {item.title}
                </div>
            </button>
        {:else}
            <TitleCell title={item.title} coverArt={item.coverArt} />
        {/if}
    </td>
    <td class="px-4 py-3">
        <div class="text-[10px] font-mono text-gray-400 dark:text-gray-500 truncate max-w-[150px] sm:max-w-xs" title={item.path}>
            {getRelativePath(item.path)}
        </div>
    </td>
    <td class="px-4 py-3">
        <span class="text-sm text-gray-500 dark:text-gray-400">
            {item.artist || ''}
        </span>
    </td>
    <td class="px-4 py-3">
        <span class="text-sm text-gray-500 dark:text-gray-400">
            {item.album || ''}
        </span>
    </td>
    <td class="px-4 py-3 text-right">
        {#if !item.isDir}
            <DurationCell duration={item.duration} />
        {/if}
    </td>
{/snippet}

{#snippet emptyState()}
    <div class="flex flex-col items-center justify-center h-64 text-gray-500">
        <Folder size={48} class="mb-4 opacity-20" />
        <p>This directory is empty</p>
    </div>
{/snippet}

<div class="flex-1 flex flex-col min-h-0 bg-white dark:bg-gray-900 rounded-2xl border border-gray-100 dark:border-gray-800 shadow-sm overflow-hidden">
    <div class="p-4 border-b border-gray-100 dark:border-gray-800 flex items-center gap-4">
        {#if history.length > 0}
            <button 
                onclick={goBack}
                class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                aria-label="Go back"
            >
                <ChevronLeft size={20} />
            </button>
        {/if}
        
        {#if history.length === 0 && musicFolders.length > 0}
            <div class="flex items-center gap-2">
                <Folder size={20} class="text-orange-600" />
                <select 
                    bind:value={selectedFolderId}
                    onchange={() => {
                        history = [];
                        fetchDirectory(selectedFolderId);
                    }}
                    class="bg-transparent text-lg font-semibold text-gray-900 dark:text-white border-none focus:ring-0 cursor-pointer outline-none appearance-none pr-8 bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20fill%3D%22none%22%20viewBox%3D%220%200%2020%2020%22%3E%3Cpath%20stroke%3D%22%236b7280%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%20d%3D%22m6%208%204%204%204-4%22%2F%3E%3C%2Fsvg%3E')] bg-[length:1.25rem_1.25rem] bg-[right_0.5rem_center] bg-no-repeat"
                >
                    {#each musicFolders as folder}
                        <option value={folder.directoryId || folder.id.toString()} class="bg-white dark:bg-gray-800">{folder.name}</option>
                    {/each}
                </select>
            </div>
        {:else}
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                {currentDirectory ? currentDirectory.name : 'Music Folders'}
            </h2>
        {/if}
    </div>

    <div class="flex-1 overflow-auto">
        {#if loading}
            <div class="flex items-center justify-center h-64">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
            </div>
        {:else if !currentDirectory}
            <div class="flex flex-col items-center justify-center h-64 text-gray-500">
                <Folder size={48} class="mb-4 opacity-20" />
                <p class="mb-4">No music folders configured</p>
                <button
                    onclick={() => navigate('/settings/folders')}
                    class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors font-medium text-sm"
                >
                    Configure Folders
                </button>
            </div>
        {:else}
            <DataTable
                data={currentDirectory.child || []}
                {loading}
                minWidth="800px"
                resizable={true}
                striped={true}
                onRowClick={handleRowClick}
                {header}
                {row}
                {emptyState}
            />
        {/if}
    </div>
</div>

<SongMetadataDrawer bind:isOpen={isDrawerOpen} songId={selectedSongId} />
