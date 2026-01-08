use crate::browser::{AlbumListOptions, Browser};
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumID3, AlbumList, AlbumList2, Artist, ArtistID3, Child, RandomSongs, SongsByGenre,
        Starred, Starred2, SubsonicResponse, SubsonicResponseBody,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsByGenreQuery {
    pub genre: String,
    #[serde(default = "default_count")]
    pub count: u64,
    #[serde(default)]
    pub offset: u64,
    pub music_folder_id: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolderQuery {
    pub music_folder_id: Option<i32>,
}

fn default_count() -> u64 {
    10
}

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
        album: albums.into_iter().map(AlbumID3::from).collect(),
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
        album: albums.into_iter().map(Child::from_album_stats).collect(),
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
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_songs_by_genre(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<SongsByGenreQuery>,
) -> impl IntoResponse {
    let songs = match browser
        .get_songs_by_genre(
            &query.genre,
            query.count,
            query.offset,
            query.music_folder_id,
        )
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
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<MusicFolderQuery>,
) -> impl IntoResponse {
    let music_folder_id = query.music_folder_id;

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
        artist: artists.into_iter().map(Artist::from).collect(),
        album: albums.into_iter().map(Child::from_album_stats).collect(),
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<MusicFolderQuery>,
) -> impl IntoResponse {
    let music_folder_id = query.music_folder_id;

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
        artist: artists.into_iter().map(ArtistID3::from).collect(),
        album: albums.into_iter().map(AlbumID3::from).collect(),
        song: songs.into_iter().map(Child::from).collect(),
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
