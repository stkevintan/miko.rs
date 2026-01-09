use crate::browser::Browser;
use crate::models::user;
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{Bookmark, Bookmarks, Child, PlayQueue, SubsonicResponse, SubsonicResponseBody},
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CreateBookmarkQuery {
    pub id: String,
    pub position: Option<i64>,
    pub comment: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteBookmarkQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SavePlayQueueQuery {
    pub current: Option<String>,
    pub position: Option<i64>,
    #[serde(default)]
    pub id: Vec<String>,
}

#[handler]
pub async fn get_bookmarks(
    browser: Data<&Arc<Browser>>,
    user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let username = &user.username;

    match browser.get_bookmarks(username).await {
        Ok(res) => {
            let bookmarks = res
                .into_iter()
                .map(|(bm, s)| Bookmark {
                    entry: s.into(),
                    position: bm.position,
                    comment: bm.comment,
                    created: bm.created_at,
                    changed: bm.updated_at,
                    username: bm.username,
                })
                .collect();

            send_response(
                SubsonicResponse::new_ok(SubsonicResponseBody::Bookmarks(Bookmarks {
                    bookmark: bookmarks,
                })),
                &params.f,
            )
        }
        Err(e) => {
            log::error!("Database error in get_bookmarks: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to fetch bookmarks".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn create_bookmark(
    browser: Data<&Arc<Browser>>,
    user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<CreateBookmarkQuery>,
) -> impl IntoResponse {
    let username = &user.username;
    let position = query.position.unwrap_or(0);

    match browser
        .create_bookmark(username, &query.id, position, query.comment.clone())
        .await
    {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in create_bookmark: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to create bookmark".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn delete_bookmark(
    browser: Data<&Arc<Browser>>,
    user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<DeleteBookmarkQuery>,
) -> impl IntoResponse {
    let username = &user.username;

    match browser.delete_bookmark(username, &query.id).await {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in delete_bookmark: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to delete bookmark".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_play_queue(
    browser: Data<&Arc<Browser>>,
    user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let username = &user.username;

    match browser.get_play_queue(username).await {
        Ok(Some((pq, songs))) => {
            let resp_pq = PlayQueue {
                current: pq.current,
                position: Some(pq.position),
                changed: Some(pq.changed),
                changed_by: Some(pq.changed_by),
                username: pq.username,
                entry: songs.into_iter().map(Child::from).collect(),
            };

            send_response(
                SubsonicResponse::new_ok(SubsonicResponseBody::PlayQueue(resp_pq)),
                &params.f,
            )
        }
        Ok(None) => {
            // Return empty queue
            let resp_pq = PlayQueue {
                current: None,
                position: None,
                changed: None,
                changed_by: None,
                username: username.to_string(),
                entry: Vec::new(),
            };
            send_response(
                SubsonicResponse::new_ok(SubsonicResponseBody::PlayQueue(resp_pq)),
                &params.f,
            )
        }
        Err(e) => {
            log::error!("Database error in get_play_queue: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to fetch play queue".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn save_play_queue(
    browser: Data<&Arc<Browser>>,
    user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<SavePlayQueueQuery>,
) -> impl IntoResponse {
    let username = &user.username;
    let client_name = params.c.as_deref().unwrap_or("Default");
    let position = query.position.unwrap_or(0);

    match browser
        .save_play_queue(
            username,
            query.0.current,
            position,
            query.0.id,
            client_name,
        )
        .await
    {
        Ok(_) => send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::None),
            &params.f,
        ),
        Err(e) => {
            log::error!("Database error in save_play_queue: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to save play queue".into()),
                &params.f,
            )
        }
    }
}
