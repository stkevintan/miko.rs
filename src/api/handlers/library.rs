use poem::{handler, web::{Data, Json, Path, Multipart}, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::models::child;
use serde::Deserialize;
use std::sync::Arc;
use lofty::prelude::*;
use lofty::tag::{ItemKey, TagItem, ItemValue};
use lofty::probe::Probe;
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType, MimeType};
use crate::service::scrape::ScrapeService;
use crate::service::tag::SongTags;

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
    Json(req): Json<ScrapeRequest>,
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
