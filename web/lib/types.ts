export interface Stats {
    songs?: number;
    albums?: number;
    artists?: number;
    genres?: number;
}

export interface SystemInfo {
    cpu_usage: number;
    memory_usage: number;
    memory_total: number;
}

export interface FolderInfo {
    label: string;
    path: string;
    song_count: number;
}

export interface ScanStatus {
    scanning: boolean;
    count: number;
    total: number;
}

export interface DashboardData {
    stats: Stats;
    system: SystemInfo;
    folders: FolderInfo[];
}

export interface UserProfile {
    username: string;
    email?: string;
    adminRole: boolean;
}

export interface Song {
    id: string;
    parent?: string;
    isDir: boolean;
    title: string;
    album?: string;
    artist?: string;
    track?: number;
    year?: number;
    genre?: string;
    coverArt?: string;
    size?: number;
    contentType?: string;
    suffix?: string;
    duration?: number;
    bitRate?: number;
    path?: string;
    playCount?: number;
    created?: string;
    albumId?: string;
    artistId?: string;
    type?: string;
}

export interface SearchResult3 {
    song?: Song[];
    album?: any[];
    artist?: any[];
}

export interface SubsonicResponse {
    status: 'ok' | 'failed';
    version: string;
    searchResult3?: SearchResult3;
}
