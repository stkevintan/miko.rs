import axios, {
    type AxiosResponse,
    type InternalAxiosRequestConfig,
} from 'axios';
import { authStore } from './auth.svelte';

export const api = axios.create({
    baseURL: '/api',
});

api.interceptors.request.use((config) => setAuthToken(config));

// allow list for 401 without logout
const allowList = ['/profile'];
function match(url: string | undefined) {
    if (!url) return false;
    try {
        const path = new URL(url).pathname;
        return allowList.includes(path);
    } catch {
        return false;
    }
}

api.interceptors.response.use(
    (response) => response,
    (error) => {
        const resp = error.response as AxiosResponse | undefined;
        if (resp?.status === 401 && !match(resp.config.url)) {
            authStore.logout();
        }
        return Promise.reject(error);
    },
);

// export const subsonic = axios.create({
//     baseURL: '/rest',
// });

// subsonic.interceptors.request.use((config) => {
//     config.params = {
//         ...config.params,
//         f: 'json',
//         v: '1.16.1',
//         c: 'miko-web',
//     };
//     return setAuthToken(config);
// });

api.interceptors.response.use(
    (response) => {
        const contentType = response.headers['content-type'];
        if (
            typeof contentType === 'string' &&
            contentType.includes('application/json')
        ) {
            const data = response.data['subsonic-response'];
            if (data) {
                if (data.status === 'failed') {
                    const subsonicError = data.error || {
                        code: 0,
                        message: 'Unknown Subsonic error',
                    };
                    const error = new Error(subsonicError.message);
                    (error as any).code = subsonicError.code;
                    (error as any).status = data.status;
                    return Promise.reject(error);
                }
                return { ...response, data };
            }
        }
        return response;
    },
    (error) => {
        return Promise.reject(error);
    },
);

function setAuthToken(config: InternalAxiosRequestConfig) {
    const token = localStorage.getItem('token');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
}

export async function getCoverArtUrl(
    id: string,
    signal?: AbortSignal,
): Promise<string> {
    const response = await api.get('/getCoverArt', {
        params: { id },
        responseType: 'blob',
        signal,
    });

    return URL.createObjectURL(response.data as Blob);
}
