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
    id: number;
    label: string;
    path: string;
    song_count: number;
}

export interface ScanStatus {
    scanning: boolean;
    count: number;
    total: number;
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

export interface ArtistReference {
    id: string;
    name: string;
    coverArt?: string;
    albumCount?: number;
    averageRating?: number;
}

export interface AlbumReference {
    id: string;
    name: string;
    artist?: string;
    artistId?: string;
    coverArt?: string;
    songCount: number;
    duration: number;
    playCount?: number;
    created: string;
    starred?: string;
    year?: number;
    genre?: string;
}

export interface AlbumList2 {
    album: AlbumReference[];
}

export interface GenreReference {
    value: string;
    songCount: number;
    albumCount: number;
}

export interface GenresResponse {
    genre: GenreReference[];
}

export interface SubsonicUser {
    username: string;
    email?: string;
    adminRole: boolean;
    settingsRole: boolean;
    downloadRole: boolean;
    uploadRole: boolean;
    playlistRole: boolean;
    coverArtRole: boolean;
    commentRole: boolean;
    podcastRole: boolean;
    streamRole: boolean;
    jukeboxRole: boolean;
    shareRole: boolean;
    videoConversionRole: boolean;
    folder: number[];
}

export interface UsersResponse {
    user: SubsonicUser[];
}
export interface SearchResult3 {
    song?: Song[];
    album?: AlbumReference[];
    artist?: ArtistReference[];
}

export interface SubsonicResponse {
    status: 'ok' | 'failed';
    version: string;
    searchResult3?: SearchResult3;
    albumList2?: AlbumList2;
    genres?: GenresResponse;
    users?: UsersResponse;
}
