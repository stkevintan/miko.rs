import { push } from 'svelte-spa-router';
import api from './api';
import type { UserProfile } from './types';

class AuthStore {
    user = $state<UserProfile | null>(null);
    loading = $state(false);

    async fetchProfile() {
        if (this.user) return;
        
        const token = localStorage.getItem('token');
        if (!token) return;

        this.loading = true;
        try {
            const resp = await api.get('/me');
            this.user = resp.data;
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
