import { createRouter } from 'sv-router';
import Login from './routes/Login.svelte';
import Dashboard from './routes/Dashboard.svelte';
import Library from './routes/Library.svelte';
import NotFound from './routes/NotFound.svelte';
import MainLayout from './components/MainLayout.svelte';

export const { p, navigate, route, isActive } = createRouter({
    '/(login)': Login,
    layout: MainLayout,
    '/': Dashboard, // Root within layout is dashboard
    '/dashboard': Dashboard,
    '/library': Library,
    '*': NotFound,
});

export function isActive2(path: string): boolean {
    return isActive(path as any);
}