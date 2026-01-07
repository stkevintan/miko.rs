use crate::config::Config;
use crate::models::child;
use crate::scanner::utils::get_cover_cache_dir;
use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::{
    Lyrics, LyricsLine, LyricsList, StructuredLyrics, SubsonicResponse, SubsonicResponseBody,
};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Query, StaticFileRequest},
    IntoResponse,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[handler]
pub async fn stream(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = crate::get_id_or_error!(query, params);

    let song = match child::Entity::find()
        .filter(child::Column::Id.eq(id))
        .select_only()
        .column(child::Column::Path)
        .column(child::Column::IsDir)
        .one(*db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Song not found".into()),
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

    if song.is_dir {
        return send_response(
            SubsonicResponse::new_error(70, "ID is a directory".into()),
            &params.f,
        );
    }

    let path = Path::new(&song.path);
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
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = crate::get_id_or_error!(query, params);

    let song = match child::Entity::find()
        .filter(child::Column::Id.eq(id))
        .select_only()
        .column(child::Column::Path)
        .one(*db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Song not found".into()),
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

    let path = Path::new(&song.path);
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

    match file_req.create_response(path, false, false) {
        Ok(resp) => {
            let mut res = resp.into_response();
            res.headers_mut().insert(
                poem::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename)
                    .parse()
                    .unwrap(),
            );
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
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let id = crate::get_id_or_error!(query, params);

    let cover_art = if id.starts_with("al-") || id.starts_with("ar-") {
        id.to_string()
    } else {
        match child::Entity::find()
            .filter(child::Column::Id.eq(id))
            .select_only()
            .column(child::Column::CoverArt)
            .one(*db)
            .await
        {
            Ok(Some(s)) => s.cover_art.unwrap_or_default(),
            _ => {
                return send_response(
                    SubsonicResponse::new_error(70, "Cover art not found".into()),
                    &params.f,
                )
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
    let cache_path = cache_dir.join(&cover_art);

    if cache_path.exists() {
        return match file_req.create_response(&cache_path, false, false) {
            Ok(resp) => resp.into_response(),
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        };
    }

    StatusCode::NOT_FOUND.into_response()
}

#[handler]
pub async fn get_lyrics(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let artist = match query.get("artist") {
        Some(a) => a,
        None => {
            return send_response(
                SubsonicResponse::new_error(10, "Artist is required".into()),
                &params.f,
            )
        }
    };
    let title = match query.get("title") {
        Some(t) => t,
        None => {
            return send_response(
                SubsonicResponse::new_error(10, "Title is required".into()),
                &params.f,
            )
        }
    };

    let song = match child::Entity::find()
        .filter(child::Column::Artist.eq(artist))
        .filter(child::Column::Title.eq(title))
        .one(*db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            return send_response(
                SubsonicResponse::new_error(70, "Lyrics not found".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Lyrics(Lyrics {
        artist: Some(song.artist),
        title: Some(song.title),
        value: song.lyrics,
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_lyrics_by_song_id(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = crate::get_id_or_error!(query, params);

    let song = match child::Entity::find()
        .filter(child::Column::Id.eq(id))
        .one(*db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            return send_response(
                SubsonicResponse::new_error(70, "Lyrics not found".into()),
                &params.f,
            )
        }
    };

    if song.lyrics.is_empty() {
        return send_response(
            SubsonicResponse::new_error(70, "Lyrics not found".into()),
            &params.f,
        );
    }

    let mut lines = Vec::new();
    let mut synced = true;

    // regex: ^\[(\d+):(\d+)\.(\d+)\](.*)$
    let re = regex::Regex::new(r"^\[(\d+):(\d+)\.(\d+)\](.*)$").unwrap();

    for row in song.lyrics.lines() {
        let row = row.trim();
        if row.is_empty() {
            continue;
        }

        if let Some(caps) = re.captures(row) {
            let min: i32 = caps[1].parse().unwrap_or(0);
            let sec: i32 = caps[2].parse().unwrap_or(0);
            let ms_str = &caps[3];
            let mut ms: i32 = ms_str.parse().unwrap_or(0);
            if ms_str.len() == 2 {
                ms *= 10;
            }
            let text = caps[4].trim().to_string();
            let start_time = (min * 60 + sec) * 1000 + ms;

            lines.push(LyricsLine {
                start: Some(start_time),
                value: text,
            });
        } else {
            synced = false;
            lines.push(LyricsLine {
                start: None,
                value: row.to_string(),
            });
        }
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::LyricsList(LyricsList {
        structured_lyrics: vec![StructuredLyrics {
            synced,
            lang: Some("xxx".to_string()),
            display_artist: Some(song.artist),
            display_title: Some(song.title),
            lines,
        }],
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_avatar(
    config: Data<&Arc<Config>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let username = match query.get("username") {
        Some(u) => u,
        None => {
            return send_response(
                SubsonicResponse::new_error(10, "Username is required".into()),
                &params.f,
            )
        }
    };

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
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        };
    }

    StatusCode::NOT_FOUND.into_response()
}
