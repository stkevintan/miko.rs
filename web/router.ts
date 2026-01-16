import { createRouter } from 'sv-router';
import Login from './routes/Login.svelte';
import Dashboard from './routes/Dashboard.svelte';
import LibraryLayout from './routes/library/Layout.svelte';
import LibraryTracks from './routes/library/Tracks.svelte';
import LibraryAlbums from './routes/library/Albums.svelte';
import LibraryArtists from './routes/library/Artists.svelte';
import NotFound from './routes/NotFound.svelte';
import MainLayout from './components/MainLayout.svelte';

export const { p, navigate, route, isActive } = createRouter({
    '/(login)': Login,
    layout: MainLayout,
    '/': Dashboard, // Root within layout is dashboard
    '/dashboard': Dashboard,
    '/library': {
        '/': LibraryTracks,
        '/tracks': LibraryTracks,
        '/albums': LibraryAlbums,
        '/artists': LibraryArtists,
        layout: LibraryLayout,
    },
    '*': NotFound,
});

export function isActive2(path: string): boolean {
    return isActive.startsWith(path as any);
}
