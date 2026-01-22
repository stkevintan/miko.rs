<script lang="ts">
    import { type Snippet } from 'svelte';
    import { FileText, Music, Info, Layers, Fingerprint, Activity, Check, X, Upload, Search } from 'lucide-svelte';
    import type { SongTags } from '../../lib/types';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import Drawer from '../ui/Drawer.svelte';
    import DrawerHeader from '../ui/DrawerHeader.svelte';
    import DrawerSection from '../ui/DrawerSection.svelte';
    import MetaTile from '../ui/MetaTile.svelte';

    let { isOpen = $bindable(false), songId }: {
        isOpen: boolean;
        songId: string | null;
    } = $props();

    let tags = $state<SongTags | null>(null);
    let originalTags = $state<SongTags | null>(null);
    let isReviewing = $state(false);
    let loading = $state(false);
    let editingField = $state<string | null>(null);
    let editValue = $state("");
    let saving = $state(false);
    let uploadingImage = $state(false);
    let scraping = $state(false);
    let scrapingLyrics = $state(false);
    let mbidInput = $state("");
    let fileInput = $state<HTMLInputElement>();

    $effect(() => {
        if (isOpen && songId) {
            fetchTags(songId);
        } else if (!isOpen) {
            tags = null;
            originalTags = null;
            isReviewing = false;
            editingField = null;
            mbidInput = "";
        }
    });

    async function fetchTags(id: string) {
        loading = true;
        isReviewing = false;
        try {
            const response = await api.get<SongTags>(`/songs/${id}/tags`);
            tags = response.data;
            if (tags?.musicBrainzTrackId) {
                mbidInput = tags.musicBrainzTrackId;
            }
        } catch (error) {
            console.error('Failed to fetch song tags:', error);
        } finally {
            loading = false;
        }
    }

    async function scrapeTags() {
        if (!songId || !tags) return;

        scraping = true;
        try {
            // Keep a copy of current tags for comparison/revert
            originalTags = JSON.parse(JSON.stringify(tags));

            // Build query string with mbid if provided
            const params = new URLSearchParams();
            if (mbidInput) {
                params.append('mbid', mbidInput);
            }
            const queryString = params.toString();
            const url = `/songs/${songId}/scrape-tags${queryString ? `?${queryString}` : ''}`;
            
            const response = await api.get<SongTags>(url);
            
            // Merge labels/values from scraped results into current tags
            // but don't save yet - let the user review
            const scrapedTags = response.data;
            tags = { ...tags, ...scrapedTags };
            isReviewing = true;
            toast.success('Metadata fetched from MusicBrainz. Review the changes below.');
        } catch (error) {
            console.error('Failed to scrape tags:', error);
            toast.error('Failed to fetch metadata from MusicBrainz');
        } finally {
            scraping = false;
        }
    }

    async function scrapeLyrics() {
        if (!songId || !tags) return;

        scrapingLyrics = true;
        try {
            // Keep a copy of current tags if not already reviewing
            if (!isReviewing) {
                originalTags = JSON.parse(JSON.stringify(tags));
            }

            const response = await api.get<{ lyrics: string }>(`/songs/${songId}/scrape-lyrics`);
            
            const newLyrics = response.data.lyrics;
            tags = { ...tags, lyrics: newLyrics };
            isReviewing = true;
            toast.success('Lyrics fetched from LRCLIB. Review the changes below.');
        } catch (error) {
            console.error('Failed to scrape lyrics:', error);
            toast.error('Failed to fetch lyrics from LRCLIB');
        } finally {
            scrapingLyrics = false;
        }
    }

    function cancelReview() {
        if (originalTags) {
            tags = originalTags;
            originalTags = null;
        }
        isReviewing = false;
    }

    async function saveAllTags() {
        if (!tags || !songId) return;
        
        saving = true;
        try {
            await api.post(`/songs/${songId}/tags`, tags);
            originalTags = null;
            isReviewing = false;
            toast.success('All tags saved successfully');
        } catch (error) {
            console.error('Failed to save all tags:', error);
            toast.error('Failed to save tags');
        } finally {
            saving = false;
        }
    }

    function startEditField(field: keyof SongTags, value: any) {
        // When starting to edit, enter review mode and save original tags
        if (!isReviewing) {
            originalTags = JSON.parse(JSON.stringify(tags));
            isReviewing = true;
        }
        editingField = field;
        editValue = value || "";
    }

    function applyFieldEdit(field: keyof SongTags) {
        if (!tags) return;
        
        // Update the tag value but don't save yet - just apply to local state
        tags = { ...tags, [field]: editValue };
        editingField = null;
    }

    async function handleImageUpload(e: Event) {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (!file || !songId) return;

        // Maximum allowed image size: 10MB (must match backend limit)
        const MAX_IMAGE_SIZE = 10 * 1024 * 1024;

        // Validate file size before uploading
        if (file.size > MAX_IMAGE_SIZE) {
            const sizeMB = (file.size / (1024 * 1024)).toFixed(2);
            toast.error(`Image too large (${sizeMB}MB). Maximum size is 10MB`);
            // Clear the file input
            if (e.target) {
                (e.target as HTMLInputElement).value = '';
            }
            return;
        }

        uploadingImage = true;
        const formData = new FormData();
        formData.append('image', file);

        try {
            await api.post(`/songs/${songId}/cover`, formData, {
                headers: {
                    'Content-Type': 'multipart/form-data'
                }
            });
            // Reload tags to get new frontCover URL
            await fetchTags(songId);
            toast.success('Cover art updated');
        } catch (error) {
            console.error('Failed to upload cover art:', error);
            toast.error('Failed to update cover art');
        } finally {
            uploadingImage = false;
        }
    }

    function close() {
        isOpen = false;
    }

    function formatDuration(seconds: number | undefined) {
        if (!seconds) return '--:--';
        const min = Math.floor(seconds / 60);
        const sec = Math.floor(seconds % 60);
        return `${min}:${sec.toString().padStart(2, '0')}`;
    }
</script>

{#snippet editableField(label: string, field: keyof SongTags, value: any, type: "text" | "number" = "text")}
    {@const isChanged = isReviewing && originalTags && JSON.stringify(originalTags[field]) !== JSON.stringify(tags?.[field])}
    <div class="flex items-start gap-4 group/field min-h-[32px]">
        <div class="w-24 text-xs text-gray-500 font-medium pt-1.5">{label}</div>
        {#if editingField === field}
            <div class="flex-1 flex gap-2">
                <input 
                    {type}
                    bind:value={editValue}
                    class="flex-1 text-sm font-semibold bg-white dark:bg-gray-800 border border-orange-500 rounded px-2 py-1 outline-none focus:ring-1 focus:ring-orange-500 dark:text-gray-200"
                    onkeydown={(e) => {
                        if (e.key === 'Enter') applyFieldEdit(field);
                        if (e.key === 'Escape') editingField = null;
                    }}
                    disabled={saving}
                />
                <div class="flex gap-1">
                    <button 
                        disabled={saving}
                        onclick={() => applyFieldEdit(field)} 
                        class="p-1 text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors disabled:opacity-50"
                        title="Apply"
                    >
                        <Check size={16} />
                    </button>
                    <button 
                        onclick={() => editingField = null} 
                        class="p-1 text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700/50 rounded transition-colors"
                        title="Cancel"
                    >
                        <X size={16} />
                    </button>
                </div>
            </div>
        {:else}
            <div class="flex-1 flex flex-col">
                <button 
                    class="text-left text-sm font-semibold dark:text-gray-200 hover:text-orange-600 cursor-pointer transition-colors py-1 group-hover/field:translate-x-1 break-all {isChanged ? 'text-blue-600 dark:text-blue-400' : ''}"
                    onclick={() => startEditField(field, value)}
                    title="Click to edit"
                >
                    {value || 'Unknown'}
                </button>
                {#if isChanged}
                    <div class="text-[10px] text-gray-400 line-through truncate">
                        Was: {originalTags?.[field] || 'Unknown'}
                    </div>
                {/if}
            </div>
        {/if}
    </div>
{/snippet}

{#snippet editableTextarea(label: string, field: keyof SongTags, value: any, icon: any, action?: Snippet)}
    {@const isChanged = isReviewing && originalTags && JSON.stringify(originalTags[field]) !== JSON.stringify(tags?.[field])}
    <DrawerSection title={label} {icon} {action}>
        {#if editingField === field}
            <div class="space-y-2">
                <textarea 
                    bind:value={editValue}
                    class="w-full h-48 text-sm p-4 bg-white dark:bg-gray-800 border border-orange-500 rounded-xl outline-none focus:ring-1 focus:ring-orange-500 dark:text-gray-200"
                    placeholder="Enter {label.toLowerCase()}..."
                    disabled={saving}
                ></textarea>
                <div class="flex justify-end gap-2">
                    <button 
                        disabled={saving}
                        onclick={() => applyFieldEdit(field)} 
                        class="px-3 py-1.5 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2 text-sm disabled:opacity-50"
                    >
                        <Check size={16} /> Apply
                    </button>
                    <button 
                        onclick={() => editingField = null} 
                        class="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-sm"
                    >
                        Cancel
                    </button>
                </div>
            </div>
        {:else}
            <div 
                class="p-4 bg-gray-50 dark:bg-gray-900/40 rounded-xl text-sm leading-relaxed whitespace-pre-wrap text-gray-600 dark:text-gray-400 italic font-serif cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors {isChanged ? 'ring-1 ring-blue-500/50' : ''}"
                role="button"
                tabindex="0"
                onclick={() => startEditField(field, value)}
                onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        startEditField(field, value);
                    }
                }}
                title="Click to edit {label.toLowerCase()}"
            >
                {value || `No ${label.toLowerCase()} available. Click to add.`}
            </div>
        {/if}
    </DrawerSection>
{/snippet}

<Drawer bind:isOpen>
    {#if loading}
        <div class="h-full flex items-center justify-center">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"></div>
        </div>
    {:else if tags}
        {#snippet headerIcon()}
            <Music size={20} />
        {/snippet}
        <DrawerHeader 
            title={tags.title || 'Unknown Title'} 
            icon={headerIcon} 
            onClose={close} 
        />

        <div class="flex-1 overflow-y-auto p-6 space-y-8">
            <input
                type="file"
                accept="image/*"
                bind:this={fileInput}
                class="hidden"
                onchange={handleImageUpload}
            />

            {#if tags.frontCover}
                <button 
                    onclick={() => fileInput?.click()}
                    disabled={uploadingImage}
                    class="group/cover relative aspect-square w-full rounded-2xl overflow-hidden shadow-lg border border-gray-100 dark:border-gray-700 hover:border-orange-500 transition-all cursor-pointer"
                >
                    <img src={tags.frontCover} alt="Cover Art" class="w-full h-full object-cover group-hover/cover:scale-105 transition-transform duration-500" />
                    <div class="absolute inset-0 bg-black/40 opacity-0 group-hover/cover:opacity-100 transition-opacity flex flex-col items-center justify-center text-white gap-2">
                        {#if uploadingImage}
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
                        {:else}
                            <Upload size={32} />
                            <span class="text-sm font-bold">Replace Cover</span>
                        {/if}
                    </div>
                </button>
            {:else}
                <button
                    onclick={() => fileInput?.click()}
                    disabled={uploadingImage}
                    class="aspect-square w-full rounded-2xl bg-gray-50 dark:bg-gray-800/50 border-2 border-dashed border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center text-gray-400 hover:border-orange-500 hover:text-orange-500 transition-all cursor-pointer"
                >
                    {#if uploadingImage}
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
                    {:else}
                        <Upload size={48} class="mb-2 opacity-20" />
                        <span class="text-sm font-medium">Upload Cover Art</span>
                    {/if}
                </button>
            {/if}

            <!-- MusicBrainz Scraper -->
            {#snippet searchIcon()}
                <Search size={14} />
            {/snippet}
            <DrawerSection title="Metadata & Lyrics Lookup" icon={searchIcon}>
                <div class="space-y-3">
                    <p class="text-xs text-gray-500 mb-2">
                        Fetch metadata from MusicBrainz and lyrics from LRCLIB. Review changes before applying.
                    </p>
                    <div class="flex gap-2">
                        <div class="relative flex-1">
                            <input 
                                type="text"
                                bind:value={mbidInput}
                                placeholder="Recording ID (optional)"
                                class="w-full text-xs font-mono bg-gray-50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-700/50 rounded-lg px-3 py-2 outline-none focus:ring-1 focus:ring-orange-500"
                            />
                        </div>
                        <button 
                            onclick={scrapeTags}
                            disabled={scraping}
                            class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors flex items-center gap-2 text-xs font-bold disabled:opacity-50"
                        >
                            {#if scraping}
                                <div class="animate-spin rounded-full h-3 w-3 border-b-2 border-white"></div>
                                Fetching...
                            {:else}
                                <Search size={14} />
                                Fetch Metadata
                            {/if}
                        </button>
                    </div>

                    {#if isReviewing}
                        <div class="flex gap-2 mt-2">
                            <button 
                                onclick={saveAllTags}
                                disabled={saving}
                                class="flex-1 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center justify-center gap-2 text-xs font-bold disabled:opacity-50"
                            >
                                {#if saving}
                                    <div class="animate-spin rounded-full h-3 w-3 border-b-2 border-white"></div>
                                    Saving...
                                {:else}
                                    <Check size={14} />
                                    Apply All
                                {/if}
                            </button>
                            <button 
                                onclick={cancelReview}
                                disabled={saving}
                                class="px-4 py-2 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors flex items-center justify-center gap-2 text-xs font-bold disabled:opacity-50"
                            >
                                <X size={14} />
                                Discard
                            </button>
                        </div>
                    {:else if tags?.musicBrainzTrackId}
                        <button 
                            onclick={saveAllTags}
                            disabled={saving}
                            class="w-full mt-2 px-4 py-2 border border-green-600/30 text-green-600 dark:text-green-500 rounded-lg hover:bg-green-50 dark:hover:bg-green-900/10 transition-colors flex items-center justify-center gap-2 text-xs font-bold disabled:opacity-50"
                        >
                            <Check size={14} />
                            Save All Metadata
                        </button>
                    {/if}
                </div>
            </DrawerSection>

            <!-- Basic Meta -->
            {#snippet infoIcon()}
                <Info size={14} />
            {/snippet}
            <DrawerSection title="Basic Information" icon={infoIcon}>
                <div class="space-y-4">
                    {@render editableField('Title', 'title', tags.title)}
                    {@render editableField('Artist', 'artist', tags.artist)}
                    {@render editableField('Album', 'album', tags.album)}
                    {#if tags.albumArtist || editingField === 'albumArtist'}
                        {@render editableField('Album Artist', 'albumArtist', tags.albumArtist)}
                    {/if}
                    {@render editableField('Genre', 'genre', tags.genre)}
                </div>
            </DrawerSection>

            <!-- Technicals Grid -->
            {#snippet activityIcon()}
                <Activity size={14} />
            {/snippet}
            <DrawerSection title="Technical Details" icon={activityIcon}>
                <div class="grid grid-cols-2 gap-3 mb-4">
                    <MetaTile label="Duration" value={formatDuration(tags.duration)} />
                    <MetaTile label="Format" value={tags.format?.toUpperCase()} />
                    {#if tags.bitRate}
                        <MetaTile label="Bitrate" value="{tags.bitRate} kbps" />
                    {/if}
                    <MetaTile label="Source" value="File Tag" />
                </div>
                <div class="space-y-4">
                    {@render editableField('Track', 'track', tags.track, 'number')}
                    {@render editableField('Disc', 'disc', tags.disc, 'number')}
                    {@render editableField('Year', 'year', tags.year, 'number')}
                    {@render editableField('BPM', 'bpm', tags.bpm, 'number')}
                </div>
            </DrawerSection>

            <!-- Credits -->
            {#snippet layersIcon()}
                <Layers size={14} />
            {/snippet}

            {#if tags.composer || tags.conductor || tags.producer || tags.lyricist || tags.remixer || editingField}
            <DrawerSection title="Credits" icon={layersIcon}>
                <div class="space-y-3 px-1">
                    {@render editableField('Composer', 'composer', tags.composer)}
                    {@render editableField('Conductor', 'conductor', tags.conductor)}
                    {@render editableField('Producer', 'producer', tags.producer)}
                    {@render editableField('Lyricist', 'lyricist', tags.lyricist)}
                    {@render editableField('Remixer', 'remixer', tags.remixer)}
                    {@render editableField('Arranger', 'arranger', tags.arranger)}
                    {@render editableField('Engineer', 'engineer', tags.engineer)}
                    {@render editableField('Mixer', 'mixer', tags.mixer)}
                </div>
            </DrawerSection>
            {/if}


            <!-- Identifiers -->
            {#snippet fingerprintIcon()}
                <Fingerprint size={14} />
            {/snippet}
            <DrawerSection title="Identifiers" icon={fingerprintIcon}>
                <div class="space-y-2 font-mono text-[10px] bg-gray-50 dark:bg-gray-900/40 p-4 rounded-xl border border-gray-100 dark:border-gray-700/50">
                    {#if tags.isrc}<div class="flex justify-between gap-4"><span class="text-gray-500 flex-shrink-0">ISRC</span> <span class="text-gray-700 dark:text-gray-400 break-all text-right">{tags.isrc}</span></div>{/if}
                    {#if tags.barcode}<div class="flex justify-between gap-4"><span class="text-gray-500 flex-shrink-0">Barcode</span> <span class="text-gray-700 dark:text-gray-400 break-all text-right">{tags.barcode}</span></div>{/if}
                    {#if tags.label}<div class="flex justify-between gap-4"><span class="text-gray-500 flex-shrink-0">Label</span> <span class="text-gray-700 dark:text-gray-400 break-all text-right">{tags.label}</span></div>{/if}
                    {#if tags.musicBrainzTrackId}<div class="flex justify-between gap-4"><span class="text-gray-500 flex-shrink-0">MB Track ID</span> <span class="text-gray-700 dark:text-gray-400 break-all text-right">{tags.musicBrainzTrackId}</span></div>{/if}
                </div>
            </DrawerSection>

            {#snippet fileTextIcon()}
                <FileText size={14} />
            {/snippet}
            {#snippet lyricsAction()}
                <button 
                    onclick={(e) => { e.stopPropagation(); scrapeLyrics(); }}
                    disabled={scrapingLyrics}
                    class="p-1 -mr-1 text-gray-400 hover:text-orange-600 transition-colors disabled:opacity-50"
                    title="Fetch lyrics from LRCLIB"
                >
                    {#if scrapingLyrics}
                        <div class="animate-spin rounded-full h-3.5 w-3.5 border-b-2 border-orange-600"></div>
                    {:else}
                        <Search size={14} />
                    {/if}
                </button>
            {/snippet}
            {@render editableTextarea('Lyrics', 'lyrics', tags.lyrics, fileTextIcon, lyricsAction)}

            {#snippet commentIcon()}
                <Layers size={14} />
            {/snippet}
            {@render editableTextarea('Comment', 'comment', tags.comment, commentIcon)}
        </div>

        <!-- Sticky action bar for review mode -->
        {#if isReviewing}
            <div class="sticky bottom-0 left-0 right-0 bg-gradient-to-t from-white dark:from-gray-900 to-transparent p-6 pt-12 pointer-events-none">
                <div class="flex gap-3 pointer-events-auto shadow-lg rounded-xl overflow-hidden">
                    <button 
                        onclick={saveAllTags}
                        disabled={saving}
                        class="flex-1 px-6 py-4 bg-green-600 text-white hover:bg-green-700 transition-colors flex items-center justify-center gap-3 text-sm font-bold disabled:opacity-50"
                    >
                        {#if saving}
                            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                            Saving All Changes...
                        {:else}
                            <Check size={18} />
                            Save All Changes
                        {/if}
                    </button>
                    <button 
                        onclick={cancelReview}
                        disabled={saving}
                        class="px-6 py-4 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors flex items-center justify-center gap-3 text-sm font-bold disabled:opacity-50"
                    >
                        <X size={18} />
                        Discard
                    </button>
                </div>
            </div>
        {/if}
    {:else}
        <div class="flex-1 flex flex-col items-center justify-center p-6 text-center">
            <Info size={48} class="text-gray-300 mb-4" />
            <p class="text-gray-500">Failed to load song metadata</p>
            <button onclick={close} class="mt-4 px-4 py-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors">
                Close
            </button>
        </div>
    {/if}
</Drawer>

