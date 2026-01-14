import { push } from 'svelte-spa-router';
import { subsonic } from './api';
import type { UserProfile } from './types';

class AuthStore {
    user = $state<UserProfile | null>(null);
    loading = $state(false);

    async fetchProfile() {
        if (this.user) return;
        const username = localStorage.getItem('username');
        if (!username) {
            this.logout();
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
        push('/login');
    }
}

export const authStore = new AuthStore();
