use poem::{handler, web::{Data, Json, Path, Multipart}, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::models::child;
use serde::{Deserialize, Serialize};
use lofty::prelude::*;
use lofty::file::AudioFile;
use lofty::tag::{Accessor, ItemKey, TagItem, ItemValue};
use lofty::probe::Probe;
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType, MimeType};
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Default)]
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
    pub music_brainz_release_group_id: Option<String>,
    pub music_brainz_album_artist_id: Option<String>,
    pub music_brainz_work_id: Option<String>,
    pub music_brainz_release_track_id: Option<String>,
    pub acoustid_id: Option<String>,
    pub acoustid_fingerprint: Option<String>,
    pub musicip_puid: Option<String>,
}

#[handler]
pub async fn get_song_tags(
    db: Data<&DatabaseConnection>,
    Path(id): Path<String>,
) -> Result<Json<SongTags>, poem::Error> {
    let song = child::Entity::find_by_id(id)
        .one(*db)
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    if song.is_dir {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    }

    let path = std::path::Path::new(&song.path);
    if !path.exists() {
        return Err(poem::Error::from_status(StatusCode::NOT_FOUND));
    }

    let probe = Probe::open(path)
        .map_err(|e| {
            log::error!("Failed to open audio file: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let tagged_file = probe.read()
        .map_err(|e| {
            log::error!("Failed to read audio tags: {}", e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let properties = tagged_file.properties();
    
    let mut tags = SongTags {
        duration: properties.duration().as_secs() as u32,
        bit_rate: properties.audio_bitrate().unwrap_or(0),
        format: path.extension().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
        ..Default::default()
    };

    if let Some(primary_tag) = tagged_file.primary_tag() {
        // Core tags via Accessor trait
        tags.title = primary_tag.title().map(|s| s.into_owned());
        tags.artist = primary_tag.artist().map(|s| s.into_owned());
        tags.album = primary_tag.album().map(|s| s.into_owned());
        tags.year = primary_tag.year();
        tags.track = primary_tag.track();
        tags.genre = primary_tag.genre().map(|s| s.into_owned());
        tags.comment = primary_tag.comment().map(|s| s.into_owned());
        tags.disc = primary_tag.disk();

        // Helper for multi-value tags
        let get_all = |key: ItemKey| -> Option<Vec<String>> {
            let items: Vec<String> = primary_tag.get_items(&key)
                .filter_map(|i| i.value().text())
                .map(|s| s.to_string())
                .collect();
            if items.is_empty() { None } else { Some(items) }
        };

        // Helper for single string tags
        let get_one = |key: ItemKey| -> Option<String> {
            primary_tag.get(&key).and_then(|i| i.value().text()).map(|s| s.to_string())
        };

        // Multi-value support
        tags.artists = get_all(ItemKey::TrackArtist);
        tags.album_artists = get_all(ItemKey::AlbumArtist);
        tags.genres = get_all(ItemKey::Genre);

        // Extended tags
        tags.album_artist = get_one(ItemKey::AlbumArtist);
        tags.lyrics = get_one(ItemKey::Lyrics);
        tags.composer = get_one(ItemKey::Composer);
        tags.conductor = get_one(ItemKey::Conductor);
        tags.remixer = get_one(ItemKey::Remixer);
        tags.arranger = get_one(ItemKey::Arranger);
        tags.lyricist = get_one(ItemKey::Lyricist);
        tags.engineer = get_one(ItemKey::Engineer);
        tags.producer = get_one(ItemKey::Producer);
        tags.mixer = get_one(ItemKey::MixEngineer);
        tags.label = get_one(ItemKey::Label);
        tags.isrc = get_one(ItemKey::Isrc);
        tags.barcode = get_one(ItemKey::Barcode);
        tags.catalog_number = get_one(ItemKey::CatalogNumber);
        tags.bpm = get_one(ItemKey::Bpm).and_then(|s| s.parse().ok());
        tags.initial_key = get_one(ItemKey::InitialKey);
        tags.mood = get_one(ItemKey::Mood);
        tags.grouping = get_one(ItemKey::ContentGroup);
        tags.movement_number = get_one(ItemKey::MovementNumber);
        tags.movement_count = get_one(ItemKey::MovementTotal);
        tags.language = get_one(ItemKey::Language);
        tags.copyright = get_one(ItemKey::CopyrightMessage);
        tags.license = get_one(ItemKey::License);
        tags.encoded_by = get_one(ItemKey::EncodedBy);
        tags.encoder_settings = get_one(ItemKey::EncoderSettings);

        // MusicBrainz
        tags.music_brainz_track_id = get_one(ItemKey::MusicBrainzTrackId);
        tags.music_brainz_artist_id = get_one(ItemKey::MusicBrainzArtistId);
        tags.music_brainz_release_group_id = get_one(ItemKey::MusicBrainzReleaseGroupId);

        // Extract front cover
        for picture in primary_tag.pictures() {
            if picture.pic_type() == lofty::picture::PictureType::CoverFront {
                let base64_data = general_purpose::STANDARD.encode(picture.data());
                let mime_type = picture.mime_type().map(|m| m.as_str()).unwrap_or("image/jpeg");
                tags.front_cover = Some(format!("data:{};base64,{}", mime_type, base64_data));
                break;
            }
        }
    } else {
        // Fallback to title from database if no tags found
        tags.title = Some(song.title);
    }

    Ok(Json(tags))
}

#[handler]
pub async fn update_song_tags(
    db: Data<&DatabaseConnection>,
    Path(id): Path<String>,
    Json(new_tags): Json<SongTags>,
) -> Result<StatusCode, poem::Error> {
    let song = child::Entity::find_by_id(id)
        .one(*db)
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    if song.is_dir {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    }

    let path_str = song.path.clone();
    
    tokio::task::spawn_blocking(move || -> Result<(), poem::Error> {
        let path = std::path::Path::new(&path_str);
        if !path.exists() {
            return Err(poem::Error::from_status(StatusCode::NOT_FOUND));
        }

        let mut tagged_file = Probe::open(path)
            .map_err(|e| {
                log::error!("Failed to open audio file: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .read()
            .map_err(|e| {
                log::error!("Failed to read audio tags: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        if tagged_file.primary_tag_mut().is_none() {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
        }
        
        let tag = tagged_file.primary_tag_mut().unwrap();

        if let Some(title) = new_tags.title { tag.set_title(title); }
        if let Some(artist) = new_tags.artist { tag.set_artist(artist); }
        if let Some(album) = new_tags.album { tag.set_album(album); }
        if let Some(year) = new_tags.year { tag.set_year(year); }
        if let Some(track) = new_tags.track { tag.set_track(track); }
        if let Some(genre) = new_tags.genre { tag.set_genre(genre); }
        if let Some(comment) = new_tags.comment { tag.set_comment(comment); }
        if let Some(album_artist) = new_tags.album_artist { tag.insert(TagItem::new(ItemKey::AlbumArtist, ItemValue::Text(album_artist))); }
        
        if let Some(lyrics) = new_tags.lyrics {
            tag.insert(TagItem::new(ItemKey::Lyrics, ItemValue::Text(lyrics)));
        }

        if let Some(composer) = new_tags.composer { tag.insert(TagItem::new(ItemKey::Composer, ItemValue::Text(composer))); }
        if let Some(conductor) = new_tags.conductor { tag.insert(TagItem::new(ItemKey::Conductor, ItemValue::Text(conductor))); }
        if let Some(producer) = new_tags.producer { tag.insert(TagItem::new(ItemKey::Producer, ItemValue::Text(producer))); }
        if let Some(lyricist) = new_tags.lyricist { tag.insert(TagItem::new(ItemKey::Lyricist, ItemValue::Text(lyricist))); }
        if let Some(remixer) = new_tags.remixer { tag.insert(TagItem::new(ItemKey::Remixer, ItemValue::Text(remixer))); }
        if let Some(arranger) = new_tags.arranger { tag.insert(TagItem::new(ItemKey::Arranger, ItemValue::Text(arranger))); }
        if let Some(engineer) = new_tags.engineer { tag.insert(TagItem::new(ItemKey::Engineer, ItemValue::Text(engineer))); }
        if let Some(mixer) = new_tags.mixer { tag.insert(TagItem::new(ItemKey::MixEngineer, ItemValue::Text(mixer))); }
        if let Some(bpm) = new_tags.bpm { tag.insert(TagItem::new(ItemKey::Bpm, ItemValue::Text(bpm.to_string()))); }

        tag.save_to_path(path, WriteOptions::default()).map_err(|e| {
            log::error!("Failed to save tags to {}: {}", path.display(), e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        Ok(())
    }).await.map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))??;

    Ok(StatusCode::OK)
}

#[handler]
pub async fn update_song_cover(
    db: Data<&DatabaseConnection>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, poem::Error> {
    let song = child::Entity::find_by_id(id)
        .one(*db)
        .await
        .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
        .ok_or_else(|| poem::Error::from_status(StatusCode::NOT_FOUND))?;

    if song.is_dir {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    }

    let mut image_data = Vec::new();
    let mut mime_type = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().map(|n| n.to_string());
        if name == Some("image".to_string()) {
            mime_type = field.content_type().unwrap_or("image/jpeg").to_string();
            image_data = field.bytes().await.map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?.to_vec();
            break;
        }
    }

    if image_data.is_empty() {
        return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
    }

    let path_str = song.path.clone();
    
    tokio::task::spawn_blocking(move || -> Result<(), poem::Error> {
        let path = std::path::Path::new(&path_str);
        let mut tagged_file = Probe::open(path)
            .map_err(|e| {
                log::error!("Failed to open audio file: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .read()
            .map_err(|e| {
                log::error!("Failed to read audio tags: {}", e);
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        if tagged_file.primary_tag_mut().is_none() {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
        }
        
        let tag = tagged_file.primary_tag_mut().unwrap();

        // Convert mime type string to lofty::picture::MimeType
        let lofty_mime_type = match mime_type.as_str() {
            "image/jpeg" => MimeType::Jpeg,
            "image/png" => MimeType::Png,
            "image/gif" => MimeType::Gif,
            "image/bmp" => MimeType::Bmp,
            "image/tiff" => MimeType::Tiff,
            _ => MimeType::Unknown(mime_type),
        };

        let picture = Picture::new_unchecked(
            PictureType::CoverFront,
            Some(lofty_mime_type),
            None,
            image_data,
        );

        // Remove existing front covers
        tag.remove_picture_type(PictureType::CoverFront);
        tag.push_picture(picture);

        tag.save_to_path(path, WriteOptions::default()).map_err(|e| {
            log::error!("Failed to save cover to {}: {}", path.display(), e);
            poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        Ok(())
    }).await.map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))??;

    Ok(StatusCode::OK)
}
