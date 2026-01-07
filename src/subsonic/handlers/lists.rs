use crate::browser::{
    map_album_to_child, map_album_to_id3, map_artist_with_stats_to_id3,
    map_artist_with_stats_to_subsonic, map_child_to_subsonic, AlbumListOptions, Browser,
};
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumList, AlbumList2, RandomSongs, SongsByGenre, Starred, Starred2, SubsonicResponse,
        SubsonicResponseBody,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;

#[handler]
pub async fn get_album_list2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let albums = match browser.get_albums(list_params.0).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch albums".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumList2(AlbumList2 {
        album: albums.into_iter().map(map_album_to_id3).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_list(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let albums = match browser.get_albums(list_params.0).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch albums".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumList(AlbumList {
        album: albums.into_iter().map(map_album_to_child).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_random_songs(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let songs = match browser.get_random_songs(list_params.0).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch songs".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::RandomSongs(RandomSongs {
        song: songs.into_iter().map(map_child_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_songs_by_genre(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let genre = match query.get("genre") {
        Some(g) => g,
        None => {
            return send_response(
                SubsonicResponse::new_error(10, "Genre is required".into()),
                &params.f,
            );
        }
    };

    let count = query
        .get("count")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);
    let offset = query
        .get("offset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let music_folder_id = query
        .get("musicFolderId")
        .and_then(|v| v.parse::<i32>().ok());

    let songs = match browser
        .get_songs_by_genre(genre, count, offset, music_folder_id)
        .await
    {
        Ok(s) => s,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch songs".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SongsByGenre(SongsByGenre {
        song: songs.into_iter().map(map_child_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let music_folder_id = query
        .get("musicFolderId")
        .and_then(|v| v.parse::<i32>().ok());

    let (artists, albums, songs) = match browser.get_starred_items(music_folder_id).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch starred items".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Starred(Starred {
        artist: artists
            .into_iter()
            .map(map_artist_with_stats_to_subsonic)
            .collect(),
        album: albums.into_iter().map(map_album_to_child).collect(),
        song: songs.into_iter().map(map_child_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let music_folder_id = query
        .get("musicFolderId")
        .and_then(|v| v.parse::<i32>().ok());

    let (artists, albums, songs) = match browser.get_starred_items(music_folder_id).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch starred items".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Starred2(Starred2 {
        artist: artists
            .into_iter()
            .map(map_artist_with_stats_to_id3)
            .collect(),
        album: albums.into_iter().map(map_album_to_id3).collect(),
        song: songs.into_iter().map(map_child_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_now_playing(params: Query<SubsonicParams>) -> impl IntoResponse {
    // For now, return empty now playing list since we don't have a shared state yet
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::NowPlaying(
        crate::subsonic::models::NowPlaying { entry: Vec::new() },
    ));

    send_response(resp, &params.f)
}
