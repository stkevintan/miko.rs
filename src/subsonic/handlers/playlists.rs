use crate::models::user;
use crate::service::Service;
use crate::service::playlists::UpdatePlaylistOptions;
use crate::subsonic::common::{
    deserialize_optional_bool, deserialize_vec, send_response, SubsonicParams,
};
use crate::subsonic::models::{Playlist, Playlists, SubsonicResponse, SubsonicResponseBody};
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
    service: Data<&Arc<Service>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<GetPlaylistsParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    let target_username = query.username.as_deref().unwrap_or(username);

    match service.get_playlists(username, target_username).await {
        Ok(playlists) => {
            let playlists: Vec<_> = playlists.into_iter().map(Playlist::from).collect();
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
    service: Data<&Arc<Service>>,
    params: Data<&SubsonicParams>,
    query: Query<GetPlaylistParams>,
) -> impl IntoResponse {
    match service.get_playlist(query.id).await {
        Ok(Some(playlist)) => {
            let subsonic_playlist = Playlist::from(playlist);
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
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub song_id: Vec<String>,
}

#[handler]
pub async fn create_playlist(
    service: Data<&Arc<Service>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<CreatePlaylistParams>,
) -> impl IntoResponse {
    let name = query.name.clone();
    let owner = current_user.username.clone();
    let song_ids = query.song_id.clone();

    match service.create_playlist(name, owner, song_ids).await {
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
    #[serde(default, deserialize_with = "deserialize_optional_bool")]
    pub public: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub song_id_to_add: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec")]
    pub song_index_to_remove: Vec<i32>,
}

#[handler]
pub async fn update_playlist(
    service: Data<&Arc<Service>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<UpdatePlaylistParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    let opts = UpdatePlaylistOptions {
        name: query.name.clone(),
        comment: query.comment.clone(),
        public: query.public,
        song_ids_to_add: query.song_id_to_add.clone(),
        song_indices_to_remove: query.song_index_to_remove.clone(),
    };

    match service
        .update_playlist(query.playlist_id, username, opts)
        .await
    {
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
    service: Data<&Arc<Service>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<DeletePlaylistParams>,
) -> impl IntoResponse {
    let username = &current_user.username;
    match service.delete_playlist(query.id, username).await {
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
