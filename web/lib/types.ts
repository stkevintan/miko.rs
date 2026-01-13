export interface Stats {
    songs: number;
    albums: number;
    artists: number;
    playlists: number;
}

export interface SystemInfo {
    cpu_usage: number;
    memory_usage: number;
    memory_total: number;
}

export interface NowPlayingInfo {
    username: string;
    player_name: string;
    song_title: string | null;
    artist_name: string | null;
    album_name: string | null;
    album_id: string | null;
    cover_art: string | null;
    updated_at: string;
}

export interface DashboardData {
    stats: Stats;
    system: SystemInfo;
    now_playing: NowPlayingInfo[];
}

export interface UserProfile {
    username: string;
    email?: string;
    roles: string[];
}
