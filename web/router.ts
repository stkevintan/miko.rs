import { createRouter } from 'sv-router';
import Login from './routes/Login.svelte';
import Dashboard from './routes/Dashboard.svelte';
import LibraryLayout from './routes/library/Layout.svelte';
import LibraryTracks from './routes/library/Tracks.svelte';
import LibraryAlbums from './routes/library/Albums.svelte';
import LibraryArtists from './routes/library/Artists.svelte';
import LibraryGenres from './routes/library/Genres.svelte';
import LibraryFolders from './routes/Folders.svelte';
import SettingsLayout from './routes/settings/Layout.svelte';
import SettingsProfile from './routes/settings/Profile.svelte';
import SettingsFolders from './routes/settings/Folders.svelte';
import SettingsConnections from './routes/settings/Connections.svelte';
import SettingsUsers from './routes/settings/Users.svelte';
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
        '/genres': LibraryGenres,
        layout: LibraryLayout,
    },
    '/folders': LibraryFolders,
    '/settings': {
        '/': SettingsProfile,
        '/profile': SettingsProfile,
        '/folders': SettingsFolders,
        '/connections': SettingsConnections,
        '/users': SettingsUsers,
        layout: SettingsLayout,
    },
    '*': NotFound,
});

export function isActive2(path: string): boolean {
    return isActive.startsWith(path as any);
}
