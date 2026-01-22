use serde::{Deserialize, Serialize};
use lofty::prelude::*;
use lofty::tag::{Accessor, ItemKey};
use lofty::probe::Probe;
use base64::{Engine as _, engine::general_purpose};
use std::path::Path;
use anyhow::Result;

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongTags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub artists: Option<Vec<String>>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub album_artists: Option<Vec<String>>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub genres: Option<Vec<String>>,
    pub lyrics: Option<String>,
    pub comment: Option<String>,
    pub duration: u32,
    pub bit_rate: u32,
    pub format: String,
    pub front_cover: Option<String>,
    // Extended tags
    pub composer: Option<String>,
    pub conductor: Option<String>,
    pub remixer: Option<String>,
    pub arranger: Option<String>,
    pub lyricist: Option<String>,
    pub engineer: Option<String>,
    pub producer: Option<String>,
    pub mixer: Option<String>,
    pub dj_mixer: Option<String>,
    pub label: Option<String>,
    pub isrc: Option<String>,
    pub barcode: Option<String>,
    pub asin: Option<String>,
    pub catalog_number: Option<String>,
    pub bpm: Option<u32>,
    pub initial_key: Option<String>,
    pub mood: Option<String>,
    pub grouping: Option<String>,
    pub movement_name: Option<String>,
    pub movement_number: Option<String>,
    pub movement_count: Option<String>,
    pub work: Option<String>,
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    pub encoded_by: Option<String>,
    pub encoder_settings: Option<String>,
    // MusicBrainz/AcoustID
    pub music_brainz_track_id: Option<String>,
    pub music_brainz_album_id: Option<String>,
    pub music_brainz_artist_id: Option<String>,
    pub music_brainz_album_artist_id: Option<String>,
    pub music_brainz_release_group_id: Option<String>,
    pub music_brainz_work_id: Option<String>,
    pub music_brainz_release_track_id: Option<String>,
    pub acoustid_id: Option<String>,
    pub acoustid_fingerprint: Option<String>,
    pub musicip_puid: Option<String>,
}

impl SongTags {
    pub fn from_file(path: &Path) -> Result<Self> {
        let probe = Probe::open(path)?;
        let tagged_file = probe.read()?;
        let properties = tagged_file.properties();
        
        let mut tags = Self {
            duration: properties.duration().as_secs() as u32,
            bit_rate: properties.audio_bitrate().unwrap_or(0),
            format: path.extension().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
            ..Default::default()
        };

        if let Some(primary_tag) = tagged_file.primary_tag() {
            tags.populate_from_tag(primary_tag);
        }

        Ok(tags)
    }

    pub fn populate_from_tag(&mut self, tag: &lofty::tag::Tag) {
        // Core tags via Accessor trait
        self.title = tag.title().map(|s| s.into_owned());
        self.artist = tag.artist().map(|s| s.into_owned());
        self.album = tag.album().map(|s| s.into_owned());
        self.year = tag.year();
        self.track = tag.track();
        self.genre = tag.genre().map(|s| s.into_owned());
        self.comment = tag.comment().map(|s| s.into_owned());
        self.disc = tag.disk();

        // Helper for multi-value tags
        let get_all = |key: ItemKey| -> Option<Vec<String>> {
            let items: Vec<String> = tag.get_items(&key)
                .filter_map(|i| i.value().text())
                .map(|s| s.to_string())
                .collect();
            if items.is_empty() { None } else { Some(items) }
        };

        // Helper for single string tags
        let get_one = |key: ItemKey| -> Option<String> {
            tag.get(&key).and_then(|i| i.value().text()).map(|s| s.to_string())
        };

        // Multi-value support
        self.artists = get_all(ItemKey::TrackArtist);
        self.album_artists = get_all(ItemKey::AlbumArtist);
        self.genres = get_all(ItemKey::Genre);

        // Extended tags
        self.album_artist = get_one(ItemKey::AlbumArtist);
        self.lyrics = get_one(ItemKey::Lyrics);
        self.composer = get_one(ItemKey::Composer);
        self.conductor = get_one(ItemKey::Conductor);
        self.remixer = get_one(ItemKey::Remixer);
        self.arranger = get_one(ItemKey::Arranger);
        self.lyricist = get_one(ItemKey::Lyricist);
        self.engineer = get_one(ItemKey::Engineer);
        self.producer = get_one(ItemKey::Producer);
        self.mixer = get_one(ItemKey::MixEngineer);
        self.dj_mixer = get_one(ItemKey::Unknown("DJ Mixer".to_string()));
        self.label = get_one(ItemKey::Label);
        self.isrc = get_one(ItemKey::Isrc);
        self.barcode = get_one(ItemKey::Barcode);
        self.catalog_number = get_one(ItemKey::CatalogNumber);
        self.bpm = get_one(ItemKey::Bpm).and_then(|s| s.parse().ok());
        self.initial_key = get_one(ItemKey::InitialKey);
        self.mood = get_one(ItemKey::Mood);
        self.grouping = get_one(ItemKey::ContentGroup);
        self.movement_name = get_one(ItemKey::Movement);
        self.movement_number = get_one(ItemKey::MovementNumber);
        self.movement_count = get_one(ItemKey::MovementTotal);
        self.work = get_one(ItemKey::Work);
        self.language = get_one(ItemKey::Language);
        self.copyright = get_one(ItemKey::CopyrightMessage);
        self.license = get_one(ItemKey::License);
        self.encoded_by = get_one(ItemKey::EncodedBy);
        self.encoder_settings = get_one(ItemKey::EncoderSettings);

        // MusicBrainz
        self.music_brainz_track_id = get_one(ItemKey::MusicBrainzTrackId);
        self.music_brainz_album_id = get_one(ItemKey::MusicBrainzReleaseId);
        self.music_brainz_artist_id = get_one(ItemKey::MusicBrainzArtistId);
        self.music_brainz_release_group_id = get_one(ItemKey::MusicBrainzReleaseGroupId);
        self.music_brainz_album_artist_id = get_one(ItemKey::MusicBrainzReleaseArtistId);
        self.music_brainz_work_id = get_one(ItemKey::MusicBrainzWorkId);
        self.music_brainz_release_track_id = get_one(ItemKey::MusicBrainzRecordingId);

        // AcoustID / MusicIP
        self.acoustid_id = get_one(ItemKey::Unknown("Acoustid Id".to_string()));
        self.acoustid_fingerprint = get_one(ItemKey::Unknown("Acoustid Fingerprint".to_string()));
        self.musicip_puid = get_one(ItemKey::Unknown("MusicIP PUID".to_string()));

        // Extract front cover
        for picture in tag.pictures() {
            if picture.pic_type() == lofty::picture::PictureType::CoverFront {
                let base64_data = general_purpose::STANDARD.encode(picture.data());
                let mime_type = picture.mime_type().map(|m| m.as_str()).unwrap_or("image/jpeg");
                self.front_cover = Some(format!("data:{};base64,{}", mime_type, base64_data));
                break;
            }
        }
    }
}
