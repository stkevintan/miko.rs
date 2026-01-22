use crate::models::child;
use anyhow::Result;
use log::{debug, error, info, warn};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;

// Use our own MusicBrainz service instead of musicbrainz_rs
use crate::service::lyrics::LyricsService;
use crate::service::musicbrainz::MusicBrainzClient;
use crate::service::tag::SongTags;

/// Escapes Lucene special characters in a search query string.
/// Escapes individual characters that form Lucene operators and special syntax:
/// + - & | ! ( ) { } [ ] ^ " ~ * ? : \ /
/// Note: Escaping & and | prevents formation of && and || operators
fn escape_lucene(query: &str) -> String {
    let special_chars = r#"+-&|!(){}[]^"~*?:\/"#;
    let mut escaped = String::with_capacity(query.len() * 2);
    for c in query.chars() {
        if special_chars.contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

pub struct ScrapeService {
    db: DatabaseConnection,
    mb_client: Arc<MusicBrainzClient>,
    lyrics_service: Arc<LyricsService>,
}

impl ScrapeService {
    pub fn new(
        db: DatabaseConnection,
        mb_client: Arc<MusicBrainzClient>,
        lyrics_service: Arc<LyricsService>,
    ) -> Self {
        Self {
            db,
            mb_client,
            lyrics_service,
        }
    }

    pub async fn scrape_recording_tags(
        &self,
        song_id: &str,
        mbid: Option<String>,
    ) -> Result<SongTags> {
        info!("Starting metadata scrape for song_id: {}", song_id);

        let song = child::Entity::find_by_id(song_id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Song not found"))?;

        let path = std::path::Path::new(&song.path);
        let mut tags = SongTags::from_file(path).unwrap_or_default();

        let mut search_title = tags.title.clone().filter(|s| !s.trim().is_empty());
        let mut search_artist = tags.artist.clone().filter(|s| !s.trim().is_empty());

        if search_title.is_none() {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Try {artist} -- {title} or {artist} - {title} first to be more specific
                // falling back to any sequence of space/hyphen
                static FILENAME_RE_LAX: once_cell::sync::Lazy<regex::Regex> =
                    once_cell::sync::Lazy::new(|| {
                        regex::Regex::new(r"^(?P<artist>.+?)\s*[\s-]{1,}\s*(?P<title>.+)$").unwrap()
                    });

                if let Some(caps) = FILENAME_RE_LAX.captures(file_stem) {
                    if search_artist.is_none() {
                        search_artist = Some(caps["artist"].trim().to_string());
                    }
                    search_title = Some(caps["title"].trim().to_string());
                    debug!(
                        "Parsed title '{}' and artist '{:?}' from filename",
                        search_title.as_ref().unwrap(),
                        search_artist
                    );
                } else {
                    search_title = Some(file_stem.trim().to_string());
                    debug!(
                        "Parsed title '{}' from filename stem",
                        search_title.as_ref().unwrap()
                    );
                }
            }
        }

        let search_title = search_title.unwrap_or_else(|| song.title.clone());
        let search_artist = search_artist;
        let search_album = tags.album.clone().filter(|s| !s.trim().is_empty());

        let mut final_mbid = mbid;

        // If no MBID provided, try to search using tags or title
        if final_mbid.is_none() {
            debug!("No MBID provided, determining search terms from title: '{}', artist: {:?}, album: {:?}", search_title, search_artist, search_album);

            // Try 1: Title + Artist + Album
            let mut query1 = String::new();
            if !search_title.is_empty() {
                query1.push_str(&format!(
                    "recording:\"{}\"",
                    escape_lucene(&search_title)
                ));
            }
            if let Some(ref a) = search_artist {
                if !query1.is_empty() {
                    query1.push_str(" AND ");
                }
                query1.push_str(&format!("artist:\"{}\"", escape_lucene(a)));
            }
            if let Some(ref alb) = search_album {
                if !query1.is_empty() {
                    query1.push_str(" AND ");
                }
                query1.push_str(&format!("release:\"{}\"", escape_lucene(alb)));
            }

            if !query1.is_empty() {
                debug!(
                    "MusicBrainz Search Try 1 (Recording + Artist + Album): {}",
                    query1
                );

                if let Ok(recordings) = self.mb_client.search_recording(&query1).await {
                    if !recordings.is_empty() {
                        let found_id = recordings[0].id.clone();
                        debug!("Found MBID via Try 1: {}", found_id);
                        final_mbid = Some(found_id);
                    }
                }
            }

            // Try 2: Title + Artist
            if final_mbid.is_none()
                && search_album.is_some()
                && !search_title.is_empty()
                && search_artist.is_some()
            {
                let mut query2 = format!("recording:\"{}\"", escape_lucene(&search_title));
                if let Some(ref a) = search_artist {
                    query2.push_str(&format!(" AND artist:\"{}\"", escape_lucene(a)));
                }
                debug!("MusicBrainz Search Try 2 (Recording + Artist): {}", query2);

                if let Ok(recordings) = self.mb_client.search_recording(&query2).await {
                    if !recordings.is_empty() {
                        let found_id = recordings[0].id.clone();
                        debug!("Found MBID via Try 2: {}", found_id);
                        final_mbid = Some(found_id);
                    }
                }
            }

            // Try 3: Just Title
            if final_mbid.is_none() && !search_title.is_empty() {
                let query3 = format!("recording:\"{}\"", escape_lucene(&search_title));
                debug!("MusicBrainz Search Try 3 (Recording only): {}", query3);

                if let Ok(recordings) = self.mb_client.search_recording(&query3).await {
                    if !recordings.is_empty() {
                        let found_id = recordings[0].id.clone();
                        debug!("Found MBID via Try 3: {}", found_id);
                        final_mbid = Some(found_id);
                    }
                }
            }
        }

        // If we still don't have an MBID, use the database song title as a last resort search
        if final_mbid.is_none() && search_title != song.title && !song.title.is_empty() {
            let query_last = format!("recording:\"{}\"", escape_lucene(&song.title));
            debug!("Attempting last resort search with query: {}", query_last);
            if let Ok(recordings) = self.mb_client.search_recording(&query_last).await {
                if !recordings.is_empty() {
                    let found_id = recordings[0].id.clone();
                    debug!("Found MBID via last resort: {}", found_id);
                    final_mbid = Some(found_id);
                }
            }
        }

        let mbid = final_mbid.ok_or_else(|| {
            warn!("Could not find a MusicBrainz ID for song: {}", song.title);
            anyhow::anyhow!("Metadata not found in MusicBrainz")
        })?;

        info!("Fetching recording details for MBID: {}", mbid);
        let recording = self.mb_client.fetch_recording(&mbid).await.map_err(|e| {
            error!("MusicBrainz API error for MBID {}: {}", mbid, e);
            e
        })?;

        debug!(
            "Mapping MusicBrainz recording '{}' to SongTags",
            recording.title
        );
        tags.title = Some(recording.title);
        tags.music_brainz_track_id = Some(recording.id.clone());

        if let Some(isrcs) = recording.isrcs {
            if !isrcs.is_empty() {
                tags.isrc = Some(isrcs[0].clone());
            }
        }

        if let Some(artists) = recording.artist_credit {
            let artist_names: Vec<String> = artists.iter().map(|a| a.name.clone()).collect();
            tags.artist = Some(artist_names.join(", "));
            tags.artists = Some(artist_names);
            if !artists.is_empty() {
                tags.music_brainz_artist_id = Some(artists[0].artist.id.clone());
            }
        }

        if let Some(genres) = recording.genres {
            let genre_names: Vec<String> = genres.iter().map(|g| g.name.clone()).collect();
            tags.genres = Some(genre_names.clone());
            tags.genre = Some(genre_names.join(", "));
        }

        if let Some(releases) = recording.releases {
            if let Some(release) = releases.get(0) {
                tags.album = Some(release.title.clone());
                tags.music_brainz_album_id = Some(release.id.clone());
                tags.barcode = release.barcode.clone();
                tags.asin = release.asin.clone();

                if let Some(rg) = &release.release_group {
                    tags.music_brainz_release_group_id = Some(rg.id.clone());
                }

                if let Some(info) = &release.label_info {
                    if let Some(li) = info.get(0) {
                        if let Some(label) = &li.label {
                            tags.label = Some(label.name.clone());
                        }
                        tags.catalog_number = li.catalog_number.clone();
                    }
                }

                // If the release has cover art, we could potentially use it
                // For now, we prefer the local cover if it exists, or we could add a flag to fetch MB cover

                if let Some(date) = &release.date {
                    if let Ok(year) = date.split('-').next().unwrap_or("").parse::<u32>() {
                        tags.year = Some(year);
                    }
                }

                if let Some(media) = &release.media {
                    if let Some(m) = media.get(0) {
                        tags.disc = m.position;
                        if let Some(tracks) = &m.tracks {
                            // Find this recording in the tracks
                            if let Some(track) = tracks.iter().find(|t| {
                                t.recording.as_ref().map(|r| r.id.as_str()) == Some(mbid.as_str())
                            }) {
                                tags.track = track.position;
                                tags.music_brainz_release_track_id = Some(track.id.clone());
                            }
                        }
                    }
                }

                if let Some(artist_credits) = &release.artist_credit {
                    let album_artists: Vec<String> =
                        artist_credits.iter().map(|a| a.name.clone()).collect();
                    tags.album_artist = Some(album_artists.join(", "));
                    tags.album_artists = Some(album_artists);
                    if !artist_credits.is_empty() {
                        tags.music_brainz_album_artist_id =
                            Some(artist_credits[0].artist.id.clone());
                    }
                }

                // Fetch cover art from MB (overriding local tag if available)
                let has_cover = release
                    .cover_art_archive
                    .as_ref()
                    .map(|caa| caa.front)
                    .unwrap_or(false);
                if has_cover {
                    match self.mb_client.fetch_cover_art(&release.id).await {
                        Ok(Some(base64_cover)) => {
                            info!(
                                "Found cover art for release: {} (overriding local if any)",
                                release.id
                            );
                            tags.front_cover = Some(base64_cover);
                        }
                        Ok(None) => debug!(
                            "Cover art marked as present but not found for release: {}",
                            release.id
                        ),
                        Err(e) => warn!("Failed to fetch cover art: {}", e),
                    }
                }
            }
        }

        info!(
            "Successfully scraped metadata for '{}' (MBID: {})",
            song.title, mbid
        );
        Ok(tags)
    }

    pub async fn scrape_lyrics(&self, song_id: &str) -> Result<String> {
        info!("Starting lyrics scrape for song_id: {}", song_id);

        let song = child::Entity::find_by_id(song_id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Song not found"))?;

        let path = std::path::Path::new(&song.path);
        let tags = SongTags::from_file(path).unwrap_or_default();

        let title = tags.title.unwrap_or_else(|| song.title.clone());
        let artist = tags.artist;

        if let Some(artist) = artist {
            match self
                .lyrics_service
                .fetch_lyrics(&title, &artist, tags.album.as_deref(), tags.duration)
                .await
            {
                Ok(Some(lyrics)) => {
                    info!("Found lyrics for '{}' by '{}'", title, artist);
                    return Ok(lyrics);
                }
                Ok(None) => {
                    debug!("No lyrics found for '{}' by '{}'", title, artist);
                    return Err(anyhow::anyhow!("Lyrics not found"));
                }
                Err(e) => {
                    warn!("Failed to fetch lyrics: {}", e);
                    return Err(e);
                }
            }
        }

        Err(anyhow::anyhow!("Missing artist tag to search for lyrics"))
    }
}
