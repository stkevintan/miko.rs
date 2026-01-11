# miko-rs

A lightweight, high-performance Subsonic-compatible music server written in Rust.

## Features

### Subsonic API Implementation Status

This project implements the Subsonic API (version 1.16.1) to support various music clients (e.g., Symfonium, Dsub, Amuse).

#### ✅ Implemented
- **System**: `ping`, `getLicense`, `getOpenSubsonicExtensions`
- **Browsing**: `getMusicFolders`, `getIndexes`, `getMusicDirectory`, `getGenres`, `getArtists`, `getArtist`, `getAlbum`, `getSong`, `getArtistInfo`, `getArtistInfo2`, `getAlbumInfo`, `getAlbumInfo2`, `getSimilarSongs`, `getSimilarSongs2`, `getTopSongs`
- **Media Retrieval**: `stream`, `download`, `getCoverArt`, `getLyrics`, `getAvatar`
- **Playlists**: `getPlaylists`, `getPlaylist`, `createPlaylist`, `updatePlaylist`, `deletePlaylist`
- **Searching**: `search`, `search2`, `search3`
- **Scanning**: `getScanStatus`, `startScan`
- **Lists**: `getAlbumList`, `getAlbumList2`, `getRandomSongs`, `getSongsByGenre`, `getNowPlaying`, `getStarred`, `getStarred2`
- **Annotation**: `star`, `unstar`, `setRating`, `scrobble`
- **Bookmarks**: `getBookmarks`, `createBookmark`, `deleteBookmark`
- **Play Queue**: `getPlayQueue`, `savePlayQueue`
- **User Management**: `getUser`, `getUsers`

### OpenSubsonic Extensions & Enhancements
- **Multi-Artist Support**: For songs and albums, an `artists` field is included in the response. This field provides a structured list of all artists associated with the item, which is particularly useful for tracks with multiple contributors.
    - Format: `artists: [{"id": "artist_id", "name": "Artist Name"}, ...]`
- **Extended Lyrics**: Supports `getLyricsBySongId` for better lyrics compatibility with modern clients.
- **Incremental Scanning**: `startScan` is incremental by default. It only scans for new or modified files.
    - To trigger a full re-scan, append `fullScan=true` to the request.

#### ❌ Not Implemented / Planned
- **Videos**: `getVideos`, `getVideoInfo`, `hls.m3u8`, `getCaptions`
- **Chat**: `getChatMessages`, `addChatMessage`
- **User Management**: `createUser`, `updateUser`, `deleteUser`, `changePassword`
- **Podcasts**
- **Internet Radio**
- **Jukebox**

---

## Installation

### Prerequisites
- Docker and Docker Compose installed on your system.
- A directory containing your music collection.

### Using Docker Compose (Recommended)

The easiest way to run `miko-rs` is using Docker Compose. Create a `docker-compose.yml` file with the following content:

```yaml
services:
  miko-rs:
    image: ghcr.io/stkevintan/miko.rs:latest
    container_name: miko-rs
    restart: unless-stopped
    ports:
      - "8081:8081"
    environment:
      - PORT=8081
      - DATABASE_URL=sqlite:///app/data/miko.db
      - SUBSONIC_DATA_DIR=/app/data
      - SUBSONIC_MUSIC_FOLDERS=/music
      - JWT_SECRET=your_secret_key_here
      - PASSWORD_SECRET=your_password_salt_here
    volumes:
      - ./data:/app/data
      - /path/to/your/music:/music:ro
```

#### Configuration
- **PORT**: The port the server will listen on inside the container (default: `8081`).
- **DATABASE_URL**: Path to the SQLite database file (e.g., `sqlite:///app/data/miko.db`).
- **SUBSONIC_DATA_DIR**: Folder where the server stores application data (e.g., `/app/data`).
- **SUBSONIC_MUSIC_FOLDERS**: Comma-separated list of folders containing your music (e.g., `/music`).
- **JWT_SECRET**: A secret string for signing JWT tokens.
- **PASSWORD_SECRET**: A secret string used as a salt for password hashing.
- **Volumes**:
    - `/app/data`: Stores the SQLite database and search indexes.
    - `/music`: Map your local music directory to this path (read-only recommended).

### Running with Docker

You can also run it directly using the Docker CLI:

```bash
docker run -d \
  --name miko-rs \
  -p 8081:8081 \
  -e JWT_SECRET=mysecret \
  -e PASSWORD_SECRET=mysalt \
  -e SUBSONIC_DATA_DIR=/app/data \
  -e SUBSONIC_MUSIC_FOLDERS=/music \
  -v $(pwd)/data:/app/data \
  -v /path/to/your/music:/music:ro \
  ghcr.io/stkevintan/miko.rs:latest
```

---

## Development

### Local Build
To build the project locally, ensure you have the Rust toolchain installed:

```bash
cargo build --release
```

### Multi-Platform Build
A helper script is provided for cross-platform builds:

```bash
./scripts/build.sh
```
