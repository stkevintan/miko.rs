import { navigate } from '../router';
import { subsonic } from './api';
import type { UserProfile } from './types';

function parseTokenUsername(token: string | null): string | null {
    if (!token) return null;
    const parts = token.split('.');
    if (parts.length < 2) return null;
    try {
        const base64 = parts[1].replace(/-/g, '+').replace(/_/g, '/');
        const padded = base64.padEnd(
            base64.length + ((4 - (base64.length % 4)) % 4),
            '=',
        );
        const payload = JSON.parse(atob(padded));
        return typeof payload?.sub === 'string' ? payload.sub : null;
    } catch {
        return null;
    }
}
class AuthStore {
    user = $state<UserProfile | null>(null);
    loading = $state(false);

    async fetchProfile() {
        if (this.user) return;
        const token = localStorage.getItem('token');
        const username = parseTokenUsername(token);
        if (!username) {
            return;
        }

        this.loading = true;
        try {
            const resp = await subsonic.get<{ user: UserProfile }>('/getUser', {
                params: { username },
            });
            this.user = resp.data.user;
        } catch (e) {
            console.error('Failed to fetch user profile', e);
            if ((e as any).response?.status === 401) {
                this.logout();
            }
        } finally {
            this.loading = false;
        }
    }

    logout() {
        this.user = null;
        localStorage.removeItem('token');
        navigate('/login');
    }
}

export const authStore = new AuthStore();
