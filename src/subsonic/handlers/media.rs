use crate::config::Config;
use crate::models::queries::{self, FolderPathInfo};
use crate::models::{artist, child, music_folder};
use crate::scanner::utils::get_cover_cache_dir;
use crate::service::utils::parse_lrc;
use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::{
    Lyrics, LyricsLine, LyricsList, StructuredLyrics, SubsonicResponse, SubsonicResponseBody,
};

use path_clean::PathClean;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Query, StaticFileRequest},
    IntoResponse,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct IdQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct LyricsQuery {
    pub artist: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct UsernameQuery {
    pub username: String,
}

async fn get_song_path_or_error(
    db: &DatabaseConnection,
    id: &str,
    params: &SubsonicParams,
) -> Result<String, poem::Response> {
    let res = queries::song_path_info_query()
        .filter(child::Column::Id.eq(id))
        .into_model::<queries::SongPathInfo>()
        .one(db)
        .await;

    match res {
        Ok(Some(s)) => {
            // Get the assigned music folder root to prevent path traversal
            let folder = match music_folder::Entity::find_by_id(s.music_folder_id)
                .select_only()
                .column(music_folder::Column::Path)
                .into_model::<FolderPathInfo>()
                .one(db)
                .await
            {
                Ok(Some(f)) => f,
                _ => {
                    log::error!(
                        "Song {} references non-existent music folder id {}",
                        id,
                        s.music_folder_id
                    );
                    return Err(send_response(
                        SubsonicResponse::new_error(70, "Invalid library configuration".into()),
                        &params.f,
                    ));
                }
            };

            let path = Path::new(&s.path).clean();
            let root = Path::new(&folder.path).clean();

            if !path.starts_with(&root) {
                log::error!(
                    "Security: Blocked attempt to access file outside root or with traversal. ID: {}, Path: {:?}, Root: {:?}",
                    id, path, root
                );
                return Err(send_response(
                    SubsonicResponse::new_error(70, "Access denied".into()),
                    &params.f,
                ));
            }

            Ok(s.path)
        }
        Ok(None) => Err(send_response(
            SubsonicResponse::new_error(70, "Audio file not found".into()),
            &params.f,
        )),
        Err(e) => {
            log::error!("Database error: {}", e);
            Err(send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            ))
        }
    }
}

#[handler]
pub async fn stream(
    db: Data<&DatabaseConnection>,
    params: Data<&SubsonicParams>,
    query: Query<IdQuery>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = &query.id;

    let path_str = match get_song_path_or_error(*db, id, &params).await {
        Ok(p) => p,
        Err(r) => return r,
    };

    let path = Path::new(&path_str);
    if !path.exists() {
        return send_response(
            SubsonicResponse::new_error(70, "File not found on disk".into()),
            &params.f,
        );
    }

    match file_req.create_response(path, false, false) {
        Ok(resp) => resp.into_response(),
        Err(e) => {
            log::error!("Static file error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[handler]
pub async fn download(
    db: Data<&DatabaseConnection>,
    params: Data<&SubsonicParams>,
    query: Query<IdQuery>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = &query.id;

    let path_str = match get_song_path_or_error(*db, id, &params).await {
        Ok(p) => p,
        Err(r) => return r,
    };

    let path = Path::new(&path_str);
    if !path.exists() {
        return send_response(
            SubsonicResponse::new_error(70, "File not found on disk".into()),
            &params.f,
        );
    }

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("download");
    let filename = filename.replace('"', "_");

    match file_req.create_response(path, false, false) {
        Ok(resp) => {
            let mut res = resp.into_response();
            if let Ok(header) = format!("attachment; filename=\"{}\"", filename).parse() {
                res.headers_mut()
                    .insert(poem::http::header::CONTENT_DISPOSITION, header);
            } else {
                log::error!(
                    "Failed to create Content-Disposition header for filename: {}",
                    filename
                );
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            res
        }
        Err(e) => {
            log::error!("Static file error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[handler]
pub async fn get_cover_art(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    params: Data<&SubsonicParams>,
    query: Query<IdQuery>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = &query.id;

    let cover_art = if id.starts_with("al-") || id.starts_with("ar-") {
        id.to_string()
    } else {
        match child::Entity::find_by_id(id.to_string()).one(*db).await {
            Ok(Some(s)) => {
                if let Some(aid) = s.album_id {
                    format!("al-{}", aid)
                } else {
                    s.id
                }
            }
            Ok(None) => {
                return send_response(
                    SubsonicResponse::new_error(70, "Cover art not found".into()),
                    &params.f,
                )
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                return send_response(
                    SubsonicResponse::new_error(0, "Database error".into()),
                    &params.f,
                );
            }
        }
    };

    if cover_art.is_empty() {
        return send_response(
            SubsonicResponse::new_error(70, "Cover art not found".into()),
            &params.f,
        );
    }

    let cache_dir = get_cover_cache_dir(&config);
    // Sanitize to prevent path traversal
    let safe_cover_art = Path::new(&cover_art)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    if safe_cover_art.is_empty() || safe_cover_art != cover_art {
        log::warn!("Potentially malicious cover art path: {}", cover_art);
        return StatusCode::BAD_REQUEST.into_response();
    }

    let cache_path = cache_dir.join(safe_cover_art);

    if cache_path.exists() {
        return match file_req.create_response(&cache_path, false, false) {
            Ok(resp) => resp.into_response(),
            Err(e) => {
                log::error!("Error serving cover art: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    }

    StatusCode::NOT_FOUND.into_response()
}

#[handler]
pub async fn get_lyrics(
    db: Data<&DatabaseConnection>,
    params: Data<&SubsonicParams>,
    query: Query<LyricsQuery>,
) -> impl IntoResponse {
    let artist_name = &query.artist;
    let title = &query.title;

    let res = queries::lyrics_with_metadata_query()
        .filter(artist::Column::Name.eq(artist_name))
        .filter(child::Column::Title.eq(title))
        .into_model::<queries::LyricsWithMetadata>()
        .one(*db)
        .await;

    let song = match res {
        Ok(Some(s)) => s,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Lyrics not found".into()),
                &params.f,
            )
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Lyrics(Lyrics {
        artist: some_artist_name_or_default(&song.artist),
        title: Some(song.title),
        value: song.content,
    }));

    send_response(resp, &params.f)
}

fn some_artist_name_or_default(artist: &Option<String>) -> Option<String> {
    Some(artist.as_deref().unwrap_or("Unknown Artist").to_string())
}

#[handler]
pub async fn get_lyrics_by_song_id(
    db: Data<&DatabaseConnection>,
    params: Data<&SubsonicParams>,
    query: Query<IdQuery>,
) -> impl IntoResponse {
    let id = &query.id;

    let res = queries::lyrics_with_metadata_query()
        .filter(child::Column::Id.eq(id))
        .into_model::<queries::LyricsWithMetadata>()
        .one(*db)
        .await;

    let song = match res {
        Ok(Some(s)) => s,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Lyrics not found".into()),
                &params.f,
            )
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let (synced, parsed_lines) = parse_lrc(&song.content);

    let lines = parsed_lines
        .into_iter()
        .map(|(start, value)| LyricsLine { start, value })
        .collect();

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::LyricsList(LyricsList {
        structured_lyrics: vec![StructuredLyrics {
            synced,
            lang: Some("xxx".to_string()),
            display_artist: some_artist_name_or_default(&song.artist),
            display_title: Some(song.title),
            lines,
        }],
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_avatar(
    config: Data<&Arc<Config>>,
    _params: Data<&SubsonicParams>,
    query: Query<UsernameQuery>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let username = &query.username;

    let avatar_dir = Path::new(&config.subsonic.data_dir).join("avatars");
    let hash = format!("{:x}", md5::compute(username));

    let mut found_path = None;
    for ext in &[".jpg", ".png"] {
        let path = avatar_dir.join(format!("{}{}", hash, ext));
        if path.exists() {
            found_path = Some(path);
            break;
        }
    }

    if let Some(path) = found_path {
        return match file_req.create_response(&path, false, false) {
            Ok(resp) => resp.into_response(),
            Err(e) => {
                log::error!("Failed to serve avatar: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        };
    }

    StatusCode::NOT_FOUND.into_response()
}
