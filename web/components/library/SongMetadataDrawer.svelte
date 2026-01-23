<script lang="ts">
    import { type Snippet } from 'svelte';
    import {
        FileText,
        Music,
        Info,
        Layers,
        Fingerprint,
        Activity,
        Check,
        X,
        Upload,
        Search,
        RotateCcw,
        Edit2,
        ChevronRight,
    } from 'lucide-svelte';
    import type { SongTags, ScrapeCandidate } from '../../lib/types';
    import { api } from '../../lib/api';
    import { toast } from '../../lib/toast.svelte';
    import Drawer from '../ui/Drawer.svelte';
    import DrawerHeader from '../ui/DrawerHeader.svelte';
    import DrawerSection from '../ui/DrawerSection.svelte';
    import MetaTile from '../ui/MetaTile.svelte';

    let {
        isOpen = $bindable(false),
        songId,
    }: {
        isOpen: boolean;
        songId: string | null;
    } = $props();

    let tags = $state<SongTags | null>(null);
    let originalTags = $state<SongTags | null>(null);
    let isReviewing = $state(false);
    let loading = $state(false);
    let editingField = $state<string | null>(null);
    let editValue = $state('');
    let saving = $state(false);
    let scraping = $state(false);
    let scrapingLyrics = $state(false);
    let mbidInput = $state('');
    let candidates = $state<ScrapeCandidate[]>([]);
    let searching = $state(false);
    let fileInput = $state<HTMLInputElement>();

    $effect(() => {
        if (isOpen && songId) {
            fetchTags(songId);
        } else if (!isOpen) {
            tags = null;
            originalTags = null;
            isReviewing = false;
            editingField = null;
            mbidInput = '';
            candidates = [];
        }
    });

    async function fetchTags(id: string) {
        loading = true;
        isReviewing = false;
        candidates = [];
        try {
            const response = await api.get<SongTags>(`/songs/${id}/tags`);
            tags = response.data;
        } catch (error) {
            console.error('Failed to fetch song tags:', error);
        } finally {
            loading = false;
        }
    }

    async function searchCandidates() {
        if (!songId) return;

        searching = true;
        candidates = [];
        try {
            const params = new URLSearchParams();
            if (mbidInput) {
                params.append('query', mbidInput);
            }
            const response = await api.get<ScrapeCandidate[]>(
                `/songs/${songId}/scrape-search?${params.toString()}`,
            );
            candidates = response.data;
            if (candidates.length === 0) {
                toast.error('No results found on MusicBrainz');
            }
        } catch (error) {
            console.error('Failed to search candidates:', error);
            toast.error('Failed to search MusicBrainz');
        } finally {
            searching = false;
        }
    }

    async function scrapeTags(mbid?: string, albumMbid?: string) {
        if (!songId || !tags) return;

        scraping = true;
        try {
            // Keep a copy of current tags for comparison/revert
            if (!isReviewing) {
                originalTags = JSON.parse(JSON.stringify(tags));
            }

            // Build query string
            const params = new URLSearchParams();
            const targetMbid = mbid || mbidInput;
            if (targetMbid) params.append('mbid', targetMbid);
            if (albumMbid) params.append('albumMbid', albumMbid);
            
            const response = await api.get<SongTags>(`/songs/${songId}/scrape-tags?${params.toString()}`);
            const scrapedData = response.data;
            
            // In Svelte 5, updating the object via a spread of a snapshot is the safest way to ensure reactivity
            // and avoid potential proxy-related merging issues.
            const updatedTags = { ...$state.snapshot(tags), ...scrapedData };
            
            // If the scraper found a new cover, it will be in scrapedData.frontCover (base64).
            // If it's null/empty, we should preserve the current one.
            if (!scrapedData.frontCover && tags.frontCover) {
                updatedTags.frontCover = tags.frontCover;
            }
            
            tags = updatedTags;
            isReviewing = true;
            candidates = []; // Clear candidates after selection
            toast.success('Metadata fetched from MusicBrainz. Review changes below.');
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

            const response = await api.get<{ lyrics: string }>(
                `/songs/${songId}/scrape-lyrics`,
            );

            const newLyrics = response.data.lyrics;
            tags = { ...tags, lyrics: newLyrics };
            isReviewing = true;
            toast.success(
                'Lyrics fetched from LRCLIB. Review the changes below.',
            );
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
        editingField = null;
        candidates = [];
    }

    async function saveAllTags() {
        if (!tags || !songId) return;

        saving = true;
        try {
            await api.post(`/songs/${songId}/tags`, tags);
            originalTags = null;
            isReviewing = false;
            editingField = null;
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
        editValue = value || '';
    }

    function applyFieldEdit(field: keyof SongTags) {
        if (!tags) return;

        // Update the tag value but don't save yet - just apply to local state
        tags = { ...tags, [field]: editValue };
        editingField = null;
    }

    function revertField(field: keyof SongTags) {
        if (!tags || !originalTags) return;
        tags = { ...tags, [field]: originalTags[field] };
    }

    async function handleImageUpload(e: Event) {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (!file || !songId || !tags) return;

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

        // Enter review mode instead of uploading immediately
        if (!isReviewing) {
            originalTags = JSON.parse(JSON.stringify(tags));
            isReviewing = true;
        }

        const reader = new FileReader();
        reader.onload = (event) => {
            const result = event.target?.result as string;
            if (result && tags) {
                tags = { ...tags, frontCover: result };
                toast.success('Cover art updated. Apply changes to save.');
            }
            // Clear the file input so it can be used again
            if (e.target) {
                (e.target as HTMLInputElement).value = '';
            }
        };
        reader.onerror = () => {
            toast.error('Failed to read image file');
        };
        reader.readAsDataURL(file);
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

    function enterReviewMode() {
        if (!isReviewing && tags) {
            originalTags = JSON.parse(JSON.stringify(tags));
            isReviewing = true;
        }
    }

    function isFieldDifferent(field: keyof SongTags) {
        if (!isReviewing || !originalTags || !tags) return false;

        const val1 = originalTags[field];
        const val2 = tags[field];

        // If both are "unknown" (null, undefined, or empty string), they are NOT different
        const isUnknown1 = val1 === null || val1 === undefined || val1 === '';
        const isUnknown2 = val2 === null || val2 === undefined || val2 === '';

        if (isUnknown1 && isUnknown2) return false;

        return JSON.stringify(val1) !== JSON.stringify(val2);
    }
</script>

{#snippet editableField(
    label: string,
    field: keyof SongTags,
    value: any,
    type: 'text' | 'number' = 'text',
)}
    {#if isReviewing || (value !== undefined && value !== null && value !== '')}
        {@const isChanged = isFieldDifferent(field)}
        <div class="flex items-start gap-4 group/field min-h-[32px]">
            <div class="w-24 text-xs text-gray-500 font-medium pt-1.5">{label}</div>
            {#if editingField === field}
                <div class="flex-1">
                    <input
                        {type}
                        bind:value={editValue}
                        class="w-full text-sm font-semibold bg-white dark:bg-gray-800 border border-orange-500 rounded px-2 py-1 outline-none focus:ring-1 focus:ring-orange-500 dark:text-gray-200"
                        onkeydown={(e) => {
                            if (e.key === 'Enter') applyFieldEdit(field);
                            if (e.key === 'Escape') editingField = null;
                        }}
                        onblur={() => applyFieldEdit(field)}
                        disabled={saving}
                    />
                </div>
            {:else}
                <div class="flex-1 flex flex-col relative group/actions">
                    <div class="flex items-center justify-between gap-2">
                        {#if isReviewing}
                            <button
                                class="text-left text-sm font-semibold dark:text-gray-200 hover:text-orange-600 cursor-pointer transition-colors py-1 group-hover/field:translate-x-1 break-all {isChanged
                                    ? 'text-blue-600 dark:text-blue-400'
                                    : ''}"
                                onclick={() => startEditField(field, value)}
                                title="Click to edit"
                            >
                                {value || 'Unknown'}
                            </button>
                        {:else}
                            <div class="text-left text-sm font-semibold dark:text-gray-200 py-1 break-all">
                                {value || 'Unknown'}
                            </div>
                        {/if}
                        {#if isChanged}
                            <button 
                                onclick={(e) => { e.stopPropagation(); revertField(field); }}
                                class="p-1 text-gray-400 hover:text-orange-600 transition-colors opacity-0 group-hover/actions:opacity-100"
                                title="Revert to original"
                            >
                                <RotateCcw size={14} />
                            </button>
                        {/if}
                    </div>
                    {#if isChanged}
                        <div
                            class="text-[10px] text-gray-400 line-through truncate"
                        >
                            Was: {originalTags?.[field] || 'Unknown'}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    {/if}
{/snippet}

{#snippet editableTextarea(
    label: string,
    field: keyof SongTags,
    value: any,
    icon: any,
    action?: Snippet,
)}
    {#if isReviewing || (value !== undefined && value !== null && value !== '')}
        {@const isChanged = isFieldDifferent(field)}
        
        {#snippet combinedAction()}
            <div class="flex items-center gap-1">
                {#if isChanged}
                    <button 
                        onclick={(e) => { e.stopPropagation(); revertField(field); }}
                        class="p-1 text-gray-400 hover:text-orange-600 transition-colors"
                        title="Revert to original"
                    >
                        <RotateCcw size={14} />
                    </button>
                {/if}
                {#if action}
                    {@render action()}
                {/if}
            </div>
        {/snippet}

        <DrawerSection title={label} {icon} action={combinedAction}>
            {#if editingField === field}
                <div class="space-y-2">
                    <textarea
                        bind:value={editValue}
                        class="w-full h-48 text-sm p-4 bg-white dark:bg-gray-800 border border-orange-500 rounded-xl outline-none focus:ring-1 focus:ring-orange-500 dark:text-gray-200"
                        placeholder="Enter {label.toLowerCase()}..."
                        onblur={() => applyFieldEdit(field)}
                        disabled={saving}
                    ></textarea>
                </div>
            {:else if isReviewing}
                <div
                    class="p-4 bg-gray-50 dark:bg-gray-900/40 rounded-xl text-sm leading-relaxed whitespace-pre-wrap text-gray-600 dark:text-gray-400 italic font-serif cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors {isChanged
                        ? 'ring-1 ring-blue-500/50'
                        : ''}"
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
            {:else}
                <div
                    class="p-4 bg-gray-50 dark:bg-gray-900/40 rounded-xl text-sm leading-relaxed whitespace-pre-wrap text-gray-600 dark:text-gray-400 italic font-serif"
                >
                    {value || `No ${label.toLowerCase()} available.`}
                </div>
            {/if}
            {#if isChanged}
                <div class="mt-2 px-4 py-2 bg-blue-50/50 dark:bg-blue-900/10 rounded-lg border border-blue-100/50 dark:border-blue-900/20">
                    <div class="text-[10px] text-blue-600 dark:text-blue-400 font-bold uppercase tracking-wider mb-1">Original Value</div>
                    <div class="text-xs text-gray-400 line-through whitespace-pre-wrap">{originalTags?.[field] || 'None'}</div>
                </div>
            {/if}
        </DrawerSection>
    {/if}
{/snippet}

<Drawer bind:isOpen>
    {#if loading}
        <div class="h-full flex items-center justify-center">
            <div
                class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-600"
            ></div>
        </div>
    {:else if tags}
        {#snippet headerIcon()}
            <Music size={20} />
        {/snippet}
        {#snippet headerAction()}
            {#if !isReviewing}
                <button
                    onclick={enterReviewMode}
                    class="p-1 text-gray-400 hover:text-orange-600 transition-colors"
                    title="Enter Review Mode"
                >
                    <Edit2 size={16} />
                </button>
            {/if}
        {/snippet}
        <DrawerHeader
            title={tags.title || 'Unknown Title'}
            icon={headerIcon}
            action={headerAction}
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
                {@const isCoverChanged = isFieldDifferent('frontCover')}
                <div class="space-y-2">
                    <button
                        onclick={() => isReviewing && fileInput?.click()}
                        class="group/cover relative aspect-square w-full rounded-2xl overflow-hidden shadow-lg border border-gray-100 dark:border-gray-700 transition-all {isReviewing
                            ? 'hover:border-orange-500 cursor-pointer'
                            : 'cursor-default'} {isCoverChanged
                            ? 'ring-4 ring-blue-500 ring-offset-4 dark:ring-offset-gray-900'
                            : ''}"
                    >
                        <img
                            src={tags.frontCover}
                            alt="Cover Art"
                            class="w-full h-full object-cover {isReviewing
                                ? 'group-hover/cover:scale-105'
                                : ''} transition-transform duration-500"
                        />
                        {#if isReviewing}
                            <div
                                class="absolute inset-0 bg-black/40 opacity-0 group-hover/cover:opacity-100 transition-opacity flex flex-col items-center justify-center text-white gap-2"
                            >
                                <Upload size={32} />
                                <span class="text-sm font-bold">Replace Cover</span>
                            </div>
                        {/if}
                    </button>
                    {#if isCoverChanged}
                        <div class="flex items-center justify-between gap-2 px-1">
                            <span class="text-[10px] font-bold text-blue-600 dark:text-blue-400 uppercase tracking-wider">Cover Changed</span>
                            <button 
                                onclick={() => revertField('frontCover')}
                                class="flex items-center gap-1.5 text-[10px] font-bold text-gray-400 hover:text-orange-600 transition-colors"
                            >
                                <RotateCcw size={12} />
                                Revert Cover
                            </button>
                        </div>
                    {/if}
                </div>
            {:else}
                <div
                    class="aspect-square w-full rounded-2xl bg-gray-50 dark:bg-gray-800/50 border-2 border-dashed border-gray-200 dark:border-gray-700 flex flex-col items-center justify-center text-gray-400 transition-all"
                >
                    {#if isReviewing}
                        <button
                            onclick={() => fileInput?.click()}
                            class="w-full h-full flex flex-col items-center justify-center hover:text-orange-500 transition-colors cursor-pointer"
                        >
                            <Upload size={48} class="mb-2 opacity-20" />
                            <span class="text-sm font-medium">Upload Cover Art</span>
                        </button>
                    {:else}
                        <Music size={48} class="opacity-10" />
                        <span class="text-xs font-medium mt-2 opacity-40 uppercase tracking-widest">No Cover Art</span>
                    {/if}
                </div>
            {/if}

            <!-- MusicBrainz Scraper -->
            {#snippet searchIcon()}
                <Search size={14} />
            {/snippet}
            <DrawerSection title="Metadata & Lyrics Lookup" icon={searchIcon}>
                <div class="space-y-4">
                    <p class="text-xs text-gray-500">
                        Fetch metadata from MusicBrainz and lyrics from LRCLIB.
                        Review changes before applying.
                    </p>
                    <div class="flex gap-2">
                        <div class="relative flex-1">
                            <input
                                type="text"
                                bind:value={mbidInput}
                                placeholder="Recording ID or Search Query"
                                class="w-full text-xs font-mono bg-gray-50 dark:bg-gray-900/40 border border-gray-100 dark:border-gray-700/50 rounded-lg px-3 py-2 outline-none focus:ring-1 focus:ring-orange-500"
                                onkeydown={(e) => {
                                    if (e.key === 'Enter') searchCandidates();
                                }}
                            />
                        </div>
                        <button
                            onclick={searchCandidates}
                            disabled={searching || scraping}
                            class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors flex items-center gap-2 text-xs font-bold disabled:opacity-50"
                        >
                            {#if searching}
                                <div
                                    class="animate-spin rounded-full h-3 w-3 border-b-2 border-white"
                                ></div>
                                Searching...
                            {:else}
                                <Search size={14} />
                                Search MB
                            {/if}
                        </button>
                    </div>

                    {#if candidates.length > 0}
                        <div class="flex items-center justify-between mb-1">
                            <span class="text-[10px] font-bold text-gray-400 uppercase tracking-wider px-1">Search Results</span>
                            <button 
                                onclick={() => candidates = []}
                                class="text-[10px] font-bold text-gray-400 hover:text-orange-600 transition-colors flex items-center gap-1 px-1"
                            >
                                <X size={12} />
                                Clear Results
                            </button>
                        </div>
                        <div class="bg-gray-50 dark:bg-gray-900/40 rounded-xl border border-gray-100 dark:border-gray-800 overflow-hidden divide-y divide-gray-100 dark:divide-gray-800">
                            {#each candidates as candidate}
                                <button
                                    onclick={() => scrapeTags(candidate.mbid, candidate.albumMbid)}
                                    disabled={scraping}
                                    class="w-full text-left p-3 hover:bg-orange-50 dark:hover:bg-orange-900/20 transition-colors group flex items-start justify-between gap-4"
                                >
                                    <div class="flex items-start gap-3 min-w-0">
                                        {#if candidate.albumMbid}
                                            <div class="w-12 h-12 rounded-lg bg-gray-100 dark:bg-gray-800 flex-shrink-0 overflow-hidden border border-gray-100 dark:border-gray-700">
                                                <img 
                                                    src="https://coverartarchive.org/release/{candidate.albumMbid}/front-250" 
                                                    alt="Cover Art"
                                                    class="w-full h-full object-cover"
                                                    onerror={(e) => (e.target as HTMLImageElement).style.display = 'none'}
                                                />
                                            </div>
                                        {:else}
                                            <div class="w-12 h-12 rounded-lg bg-gray-100 dark:bg-gray-800 flex-shrink-0 flex items-center justify-center">
                                                <Music size={20} class="text-gray-400" />
                                            </div>
                                        {/if}
                                        <div class="min-w-0 pt-0.5">
                                            <div class="text-sm font-bold text-gray-900 dark:text-gray-100 truncate">
                                                {candidate.title}
                                            </div>
                                            <div class="text-xs text-gray-500 truncate mt-0.5">
                                                {candidate.artist}
                                                {#if candidate.album}
                                                    <span class="mx-1">•</span> {candidate.album}
                                                {/if}
                                                {#if candidate.year}
                                                    <span class="mx-1">•</span> {candidate.year}
                                                {/if}
                                            </div>
                                        </div>
                                    </div>
                                    <ChevronRight size={16} class="text-gray-300 group-hover:text-orange-500 transition-colors flex-shrink-0 mt-3" />
                                </button>
                            {/each}
                        </div>
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
                    {@render editableField(
                        'Album Artist',
                        'albumArtist',
                        tags.albumArtist,
                    )}
                    {@render editableField('Genre', 'genre', tags.genre)}
                </div>
            </DrawerSection>

            <!-- Technicals Grid -->
            {#snippet activityIcon()}
                <Activity size={14} />
            {/snippet}
            <DrawerSection title="Technical Details" icon={activityIcon}>
                <div class="grid grid-cols-2 gap-3 mb-4">
                    <MetaTile
                        label="Duration"
                        value={formatDuration(tags.duration)}
                    />
                    <MetaTile
                        label="Format"
                        value={tags.format?.toUpperCase()}
                    />
                    {#if tags.bitRate}
                        <MetaTile label="Bitrate" value="{tags.bitRate} kbps" />
                    {/if}
                    <MetaTile label="Source" value="File Tag" />
                </div>
                <div class="space-y-4">
                    {@render editableField(
                        'Track',
                        'track',
                        tags.track,
                        'number',
                    )}
                    {@render editableField('Disc', 'disc', tags.disc, 'number')}
                    {@render editableField('Year', 'year', tags.year, 'number')}
                    {@render editableField('BPM', 'bpm', tags.bpm, 'number')}
                </div>
            </DrawerSection>

            <!-- Credits -->
            {#snippet layersIcon()}
                <Layers size={14} />
            {/snippet}

            {#if isReviewing || tags.composer || tags.conductor || tags.producer || tags.lyricist || tags.remixer || tags.arranger || tags.engineer || tags.mixer}
                <DrawerSection title="Credits" icon={layersIcon}>
                    <div class="space-y-3 px-1">
                        {@render editableField(
                            'Composer',
                            'composer',
                            tags.composer,
                        )}
                        {@render editableField(
                            'Conductor',
                            'conductor',
                            tags.conductor,
                        )}
                        {@render editableField(
                            'Producer',
                            'producer',
                            tags.producer,
                        )}
                        {@render editableField(
                            'Lyricist',
                            'lyricist',
                            tags.lyricist,
                        )}
                        {@render editableField(
                            'Remixer',
                            'remixer',
                            tags.remixer,
                        )}
                        {@render editableField(
                            'Arranger',
                            'arranger',
                            tags.arranger,
                        )}
                        {@render editableField(
                            'Engineer',
                            'engineer',
                            tags.engineer,
                        )}
                        {@render editableField('Mixer', 'mixer', tags.mixer)}
                    </div>
                </DrawerSection>
            {/if}

            <!-- Identifiers -->
            {#snippet fingerprintIcon()}
                <Fingerprint size={14} />
            {/snippet}
            {#if isReviewing || tags.isrc || tags.barcode || tags.label || tags.musicBrainzTrackId}
                <DrawerSection title="Identifiers" icon={fingerprintIcon}>
                    <div class="space-y-4">
                        {@render editableField('ISRC', 'isrc', tags.isrc)}
                        {@render editableField(
                            'Barcode',
                            'barcode',
                            tags.barcode,
                        )}
                        {@render editableField('Label', 'label', tags.label)}
                        {@render editableField(
                            'MB Track ID',
                            'musicBrainzTrackId',
                            tags.musicBrainzTrackId,
                        )}
                    </div>
                </DrawerSection>
            {/if}

            {#snippet fileTextIcon()}
                <FileText size={14} />
            {/snippet}
            {#snippet lyricsAction()}
                <button
                    onclick={(e) => {
                        e.stopPropagation();
                        scrapeLyrics();
                    }}
                    disabled={scrapingLyrics}
                    class="p-1 -mr-1 text-gray-400 hover:text-orange-600 transition-colors disabled:opacity-50"
                    title="Fetch lyrics from LRCLIB"
                >
                    {#if scrapingLyrics}
                        <div
                            class="animate-spin rounded-full h-3.5 w-3.5 border-b-2 border-orange-600"
                        ></div>
                    {:else}
                        <Search size={14} />
                    {/if}
                </button>
            {/snippet}
            {@render editableTextarea(
                'Lyrics',
                'lyrics',
                tags.lyrics,
                fileTextIcon,
                lyricsAction,
            )}

            {#snippet commentIcon()}
                <Layers size={14} />
            {/snippet}
            {@render editableTextarea(
                'Comment',
                'comment',
                tags.comment,
                commentIcon,
            )}
        </div>

        <!-- Sticky action bar for review mode -->
        {#if isReviewing}
            <div class="p-4 border-t border-gray-100 dark:border-gray-800 bg-white/95 dark:bg-gray-900/95 backdrop-blur-md flex gap-3 shadow-[0_-10px_20px_-5px_rgba(0,0,0,0.05)] mt-auto">
                <button
                    onclick={saveAllTags}
                    disabled={saving}
                    class="flex-1 px-4 py-2 bg-green-600 text-white rounded-xl hover:bg-green-700 active:scale-[0.98] transition-all flex items-center justify-center gap-2 text-sm font-bold shadow-lg shadow-green-600/20 disabled:opacity-50 disabled:active:scale-100"
                >
                    {#if saving}
                        <div class="animate-spin rounded-full h-4 w-4 border-2 border-white/20 border-b-white"></div>
                        <span>Saving...</span>
                    {:else}
                        <Check size={18} />
                        <span>Apply All Changes</span>
                    {/if}
                </button>
                <button
                    onclick={cancelReview}
                    disabled={saving}
                    class="px-4 py-2 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-700 active:scale-[0.98] transition-all flex items-center justify-center gap-2 text-sm font-bold disabled:opacity-50 disabled:active:scale-100"
                >
                    <X size={18} />
                    <span>Discard</span>
                </button>
            </div>
        {/if}
    {:else}
        <div
            class="flex-1 flex flex-col items-center justify-center p-6 text-center"
        >
            <Info size={48} class="text-gray-300 mb-4" />
            <p class="text-gray-500">Failed to load song metadata</p>
            <button
                onclick={close}
                class="mt-4 px-4 py-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors"
            >
                Close
            </button>
        </div>
    {/if}
</Drawer>
