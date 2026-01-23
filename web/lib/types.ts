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

export interface Directory {
    id: string;
    parent?: string;
    name: string;
    path?: string;
    child?: Song[];
}

export interface SongTags {
    title?: string;
    artist?: string;
    artists?: string[];
    album?: string;
    albumArtist?: string;
    albumArtists?: string[];
    track?: number;
    disc?: number;
    year?: number;
    genre?: string;
    genres?: string[];
    lyrics?: string;
    comment?: string;
    duration: number;
    bitRate?: number;
    format: string;
    frontCover?: string;
    // Additional tags
    composer?: string;
    conductor?: string;
    remixer?: string;
    arranger?: string;
    lyricist?: string;
    engineer?: string;
    producer?: string;
    djMixer?: string;
    mixer?: string;
    label?: string;
    isrc?: string;
    barcode?: string;
    asin?: string;
    catalogNumber?: string;
    bpm?: number;
    initialKey?: string;
    mood?: string;
    grouping?: string;
    movementName?: string;
    movementNumber?: string;
    movementCount?: string;
    work?: string;
    language?: string;
    copyright?: string;
    license?: string;
    encodedBy?: string;
    encoderSettings?: string;
    // MusicBrainz/AcoustID
    musicBrainzTrackId?: string;
    musicBrainzAlbumId?: string;
    musicBrainzArtistId?: string;
    musicBrainzReleaseGroupId?: string;
    musicBrainzAlbumArtistId?: string;
    musicBrainzWorkId?: string;
    musicBrainzReleaseTrackId?: string;
    acoustidId?: string;
    acoustidFingerprint?: string;
    musicipPuid?: string;
}

export interface ScrapeCandidate {
    mbid: string;
    title: string;
    artist: string;
    album?: string;
    albumMbid?: string;
    year?: number;
}

export interface MusicFolder {
    id: number;
    name: string;
    path?: string;
    directoryId?: string;
    songCount?: number;
}

export interface ArtistWithAlbums extends ArtistReference {
    album: AlbumReference[];
}

export interface AlbumWithSongs extends AlbumReference {
    song: Song[];
}

export interface SubsonicResponse {
    status: 'ok' | 'failed';
    version: string;
    searchResult3?: SearchResult3;
    albumList2?: AlbumList2;
    genres?: GenresResponse;
    users?: UsersResponse;
    musicFolders?: { musicFolder: MusicFolder[] };
    directory?: Directory;
    album?: AlbumWithSongs;
    artist?: ArtistWithAlbums;
}
