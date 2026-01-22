use poem::{handler, web::{Data, Json, Path, Multipart, Query}, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::models::{child, user};
use serde::Deserialize;
use std::sync::Arc;
use lofty::prelude::*;
use lofty::tag::{ItemKey, TagItem, ItemValue};
use lofty::probe::Probe;
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType, MimeType};
use crate::service::scrape::ScrapeService;
use crate::service::tag::SongTags;

const ACOUSTID_ID: &str = "Acoustid Id";
const ACOUSTID_FINGERPRINT: &str = "Acoustid Fingerprint";
const MUSICIP_PUID: &str = "MusicIP PUID";

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

    let mut tags = SongTags::from_file(path).map_err(|e| {
        log::error!("Failed to read song tags: {}", e);
        poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    // Fallback to title from database if no tags found
    if tags.title.is_none() {
        tags.title = Some(song.title);
    }

    Ok(Json(tags))
}

#[handler]
pub async fn update_song_tags(
    db: Data<&DatabaseConnection>,
    user: Data<&std::sync::Arc<user::Model>>,
    Path(id): Path<String>,
    Json(new_tags): Json<SongTags>,
) -> Result<StatusCode, poem::Error> {
    if !user.admin_role {
        return Err(poem::Error::from_status(StatusCode::FORBIDDEN));
    }

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
        
        // Additional extended tags
        if let Some(label) = new_tags.label { tag.insert(TagItem::new(ItemKey::Label, ItemValue::Text(label))); }
        if let Some(isrc) = new_tags.isrc { tag.insert(TagItem::new(ItemKey::Isrc, ItemValue::Text(isrc))); }
        if let Some(barcode) = new_tags.barcode { tag.insert(TagItem::new(ItemKey::Barcode, ItemValue::Text(barcode))); }
        if let Some(catalog_number) = new_tags.catalog_number { tag.insert(TagItem::new(ItemKey::CatalogNumber, ItemValue::Text(catalog_number))); }
        if let Some(initial_key) = new_tags.initial_key { tag.insert(TagItem::new(ItemKey::InitialKey, ItemValue::Text(initial_key))); }
        if let Some(mood) = new_tags.mood { tag.insert(TagItem::new(ItemKey::Mood, ItemValue::Text(mood))); }
        if let Some(grouping) = new_tags.grouping { tag.insert(TagItem::new(ItemKey::ContentGroup, ItemValue::Text(grouping))); }
        if let Some(movement_name) = new_tags.movement_name { tag.insert(TagItem::new(ItemKey::Movement, ItemValue::Text(movement_name))); }
        if let Some(movement_number) = new_tags.movement_number { tag.insert(TagItem::new(ItemKey::MovementNumber, ItemValue::Text(movement_number))); }
        if let Some(movement_count) = new_tags.movement_count { tag.insert(TagItem::new(ItemKey::MovementTotal, ItemValue::Text(movement_count))); }
        if let Some(work) = new_tags.work { tag.insert(TagItem::new(ItemKey::Work, ItemValue::Text(work))); }
        if let Some(language) = new_tags.language { tag.insert(TagItem::new(ItemKey::Language, ItemValue::Text(language))); }
        if let Some(copyright) = new_tags.copyright { tag.insert(TagItem::new(ItemKey::CopyrightMessage, ItemValue::Text(copyright))); }
        if let Some(license) = new_tags.license { tag.insert(TagItem::new(ItemKey::License, ItemValue::Text(license))); }
        if let Some(encoded_by) = new_tags.encoded_by { tag.insert(TagItem::new(ItemKey::EncodedBy, ItemValue::Text(encoded_by))); }
        if let Some(encoder_settings) = new_tags.encoder_settings { tag.insert(TagItem::new(ItemKey::EncoderSettings, ItemValue::Text(encoder_settings))); }
        
        // MusicBrainz IDs
        if let Some(mb_track_id) = new_tags.music_brainz_track_id { tag.insert(TagItem::new(ItemKey::MusicBrainzTrackId, ItemValue::Text(mb_track_id))); }
        if let Some(mb_album_id) = new_tags.music_brainz_album_id { tag.insert(TagItem::new(ItemKey::MusicBrainzReleaseId, ItemValue::Text(mb_album_id))); }
        if let Some(mb_artist_id) = new_tags.music_brainz_artist_id { tag.insert(TagItem::new(ItemKey::MusicBrainzArtistId, ItemValue::Text(mb_artist_id))); }
        if let Some(mb_release_group_id) = new_tags.music_brainz_release_group_id { tag.insert(TagItem::new(ItemKey::MusicBrainzReleaseGroupId, ItemValue::Text(mb_release_group_id))); }
        if let Some(mb_album_artist_id) = new_tags.music_brainz_album_artist_id { tag.insert(TagItem::new(ItemKey::MusicBrainzReleaseArtistId, ItemValue::Text(mb_album_artist_id))); }
        if let Some(mb_work_id) = new_tags.music_brainz_work_id { tag.insert(TagItem::new(ItemKey::MusicBrainzWorkId, ItemValue::Text(mb_work_id))); }
        if let Some(mb_recording_id) = new_tags.music_brainz_release_track_id { tag.insert(TagItem::new(ItemKey::MusicBrainzRecordingId, ItemValue::Text(mb_recording_id))); }

        // AcoustID / MusicIP IDs
        if let Some(acoustid_id) = new_tags.acoustid_id { tag.insert(TagItem::new(ItemKey::Unknown(ACOUSTID_ID.to_string()), ItemValue::Text(acoustid_id))); }
        if let Some(acoustid_fingerprint) = new_tags.acoustid_fingerprint { tag.insert(TagItem::new(ItemKey::Unknown(ACOUSTID_FINGERPRINT.to_string()), ItemValue::Text(acoustid_fingerprint))); }
        if let Some(musicip_puid) = new_tags.musicip_puid { tag.insert(TagItem::new(ItemKey::Unknown(MUSICIP_PUID.to_string()), ItemValue::Text(musicip_puid))); }

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
    user: Data<&std::sync::Arc<user::Model>>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, poem::Error> {
    if !user.admin_role {
        return Err(poem::Error::from_status(StatusCode::FORBIDDEN));
    }

    // Maximum allowed image size: 10MB
    const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024;

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
            
            // Validate image size
            if image_data.len() > MAX_IMAGE_SIZE {
                log::warn!("Image upload rejected: size {} exceeds maximum {}", image_data.len(), MAX_IMAGE_SIZE);
                return Err(poem::Error::from_status(StatusCode::PAYLOAD_TOO_LARGE));
            }
            
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeRequest {
    pub mbid: Option<String>,
}

#[handler]
pub async fn scrape_song_tags(
    db: Data<&DatabaseConnection>,
    mb_client: Data<&Arc<crate::service::musicbrainz::MusicBrainzClient>>,
    lyrics_service: Data<&Arc<crate::service::lyrics::LyricsService>>,
    Path(id): Path<String>,
    Query(req): Query<ScrapeRequest>,
) -> Result<Json<SongTags>, poem::Error> {
    let scrape_service = ScrapeService::new((*db).clone(), (*mb_client).clone(), (*lyrics_service).clone());
    
    let tags = scrape_service.scrape_recording_tags(&id, req.mbid)
        .await
        .map_err(|e| {
            log::error!("Scrape failed: {}", e);
            poem::Error::from_status(StatusCode::NOT_FOUND)
        })?;

    Ok(Json(tags))
}

#[handler]
pub async fn scrape_song_lyrics(
    db: Data<&DatabaseConnection>,
    mb_client: Data<&Arc<crate::service::musicbrainz::MusicBrainzClient>>,
    lyrics_service: Data<&Arc<crate::service::lyrics::LyricsService>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, poem::Error> {
    let scrape_service = ScrapeService::new((*db).clone(), (*mb_client).clone(), (*lyrics_service).clone());
    
    let lyrics = scrape_service.scrape_lyrics(&id)
        .await
        .map_err(|e| {
            log::error!("Lyrics scrape failed: {}", e);
            poem::Error::from_status(StatusCode::NOT_FOUND)
        })?;

    Ok(Json(serde_json::json!({ "lyrics": lyrics })))
}
