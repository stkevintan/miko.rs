use crate::browser::{
    map_playlist_to_subsonic, map_playlist_with_songs_to_subsonic, Browser, UpdatePlaylistOptions,
};
use crate::models::user;
use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::{Playlists, SubsonicResponse, SubsonicResponseBody};
use poem::web::{Data, Query};
use poem::{handler, IntoResponse};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetPlaylistsParams {
    pub username: Option<String>,
}

#[handler]
pub async fn get_playlists(
    browser: Data<&Arc<Browser>>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<GetPlaylistsParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    let target_username = query.username.as_deref().unwrap_or(username);

    match browser.get_playlists(username, target_username).await {
        Ok(playlists) => {
            let playlists: Vec<_> = playlists.into_iter().map(map_playlist_to_subsonic).collect();
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Playlists(Playlists {
                playlist: playlists,
            }));
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get playlists: {}", e);
            let resp = SubsonicResponse::new_error(0, "Failed to retrieve playlists".to_string());
            send_response(resp, &params.f)
        }
    }
}

#[derive(Deserialize)]
pub struct GetPlaylistParams {
    pub id: i32,
}

#[handler]
pub async fn get_playlist(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<GetPlaylistParams>,
) -> impl IntoResponse {
    match browser.get_playlist(query.id).await {
        Ok(Some(playlist)) => {
            let subsonic_playlist = map_playlist_with_songs_to_subsonic(playlist);
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Playlist(subsonic_playlist));
            send_response(resp, &params.f)
        }
        Ok(None) => {
            let resp = SubsonicResponse::new_error(70, "Playlist not found".to_string());
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get playlist: {}", e);
            let resp = SubsonicResponse::new_error(0, "Failed to retrieve playlist".to_string());
            send_response(resp, &params.f)
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaylistParams {
    pub name: String,
    pub song_id: Option<Vec<String>>,
}

#[handler]
pub async fn create_playlist(
    browser: Data<&Arc<Browser>>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<CreatePlaylistParams>,
) -> impl IntoResponse {
    let name = query.name.clone();
    let owner = current_user.username.clone();
    let song_ids = query.song_id.clone().unwrap_or_default();

    match browser.create_playlist(name, owner, song_ids).await {
        Ok(_) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::None);
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to create playlist: {}", e);
            let resp = SubsonicResponse::new_error(0, "Failed to create playlist".to_string());
            send_response(resp, &params.f)
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaylistParams {
    pub playlist_id: i32,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    pub song_id_to_add: Option<Vec<String>>,
    pub song_index_to_remove: Option<Vec<i32>>,
}

#[handler]
pub async fn update_playlist(
    browser: Data<&Arc<Browser>>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<UpdatePlaylistParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    let opts = UpdatePlaylistOptions {
        name: query.name.clone(),
        comment: query.comment.clone(),
        public: query.public,
        song_ids_to_add: query.song_id_to_add.clone().unwrap_or_default(),
        song_indices_to_remove: query.song_index_to_remove.clone().unwrap_or_default(),
    };

    match browser.update_playlist(query.playlist_id, username, opts).await {
        Ok(_) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::None);
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to update playlist: {}", e);
            let resp = SubsonicResponse::new_error(0, format!("Failed to update playlist: {}", e));
            send_response(resp, &params.f)
        }
    }
}

#[derive(Deserialize)]
pub struct DeletePlaylistParams {
    pub id: i32,
}

#[handler]
pub async fn delete_playlist(
    browser: Data<&Arc<Browser>>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<DeletePlaylistParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    match browser.delete_playlist(query.id, username).await {
        Ok(_) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::None);
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to delete playlist: {}", e);
            let resp = SubsonicResponse::new_error(0, "Failed to delete playlist".to_string());
            send_response(resp, &params.f)
        }
    }
}
