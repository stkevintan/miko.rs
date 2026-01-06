use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Tags {
    pub title: String,
    pub artist: String,
    pub artists: Vec<String>,
    pub album: String,
    pub album_artist: String,
    pub album_artists: Vec<String>,
    pub track: Option<i32>,
    pub disc: Option<i32>,
    pub year: Option<i32>,
    pub genre: String,
    pub genres: Vec<String>,
    pub lyrics: String,
    pub duration: i32,
    pub bitrate: i32,
    pub has_image: bool,
}

pub fn read(path: &Path) -> Result<Tags, anyhow::Error> {
    let tagged_file = Probe::open(path)?.read()?;

    let properties = tagged_file.properties();
    let duration = properties.duration().as_secs() as i32;
    let bitrate = properties.audio_bitrate().unwrap_or(0) as i32;

    let mut tags = Tags {
        duration,
        bitrate,
        ..Default::default()
    };

    if let Some(tag) = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
    {
        tags.title = tag.title().map(|s| s.into_owned()).unwrap_or_default();
        tags.album = tag.album().map(|s| s.into_owned()).unwrap_or_default();
        tags.genre = tag.genre().map(|s| s.into_owned()).unwrap_or_default();
        tags.track = tag.track().map(|v| v as i32);
        tags.disc = tag.disk().map(|v| v as i32);
        tags.year = tag.year().map(|v| v as i32);

        // Lyrics
        if let Some(lyrics) = tag.get_string(&lofty::tag::ItemKey::Lyrics) {
            tags.lyrics = lyrics.to_string();
        }

        // Try to get multiple artists from ARTISTS tag first
        let mut artists: Vec<String> = tag
            .get_strings(&lofty::tag::ItemKey::TrackArtists)
            .flat_map(split_tag)
            .collect();
        // Then fall back to ARTIST tag
        if artists.is_empty() {
            artists = tag
                .get_strings(&lofty::tag::ItemKey::TrackArtist)
                .flat_map(split_tag)
                .collect();
        }

        tags.artists = artists;
        tags.artist = tags.artists.join("; ");

        tags.genres = split_tag(&tags.genre);

        if let Some(item) = tag.get_string(&lofty::tag::ItemKey::AlbumArtist) {
            tags.album_artist = item.to_string();
        }

        if tags.album_artist.is_empty() {
            tags.album_artist = tags.artist.clone();
        }

        tags.album_artists = split_tag(&tags.album_artist);

        tags.has_image = !tag.pictures().is_empty();
    }

    Ok(tags)
}

fn split_tag(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn read_image(path: &Path) -> Result<Vec<u8>, anyhow::Error> {
    let tagged_file = Probe::open(path)?.read()?;
    if let Some(tag) = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
    {
        if let Some(picture) = tag.pictures().first() {
            return Ok(picture.data().to_vec());
        }
    }
    anyhow::bail!("No image found")
}
