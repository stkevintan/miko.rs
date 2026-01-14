export interface Stats {
    songs: number;
    albums: number;
    artists: number;
    genres: number;
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
