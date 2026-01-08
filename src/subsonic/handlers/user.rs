use crate::models::{user, music_folder};
use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::{
    SubsonicResponse, SubsonicResponseBody, User, Users,
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::{DatabaseConnection, EntityTrait, LoaderTrait};
use std::collections::HashMap;
use std::sync::Arc;

#[handler]
pub async fn get_users(
    db: Data<&DatabaseConnection>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    if !current_user.admin_role {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let users = match user::Entity::find().all(*db).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let folders = match users.load_many(music_folder::Entity, *db).await {
        Ok(f) => f,
        Err(e) => {
            log::error!("Database error loading folders: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let subsonic_users: Vec<User> = users
        .into_iter()
        .zip(folders.into_iter())
        .map(|(u, f)| {
            User::from_db(
                u,
                f.into_iter().map(|mf| mf.id).collect(),
            )
        })
        .collect();

    send_response(
        SubsonicResponse::new_ok(SubsonicResponseBody::Users(Users {
            user: subsonic_users,
        })),
        &params.f,
    )
}

#[handler]
pub async fn get_user(
    db: Data<&DatabaseConnection>,
    current_user: Data<&Arc<user::Model>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
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
    if !current_user.admin_role && current_user.username != *username {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let user = match user::Entity::find_by_id(username.to_string())
        .find_with_related(music_folder::Entity)
        .all(*db)
        .await
    {
        Ok(res) => {
            if let Some((u, f)) = res.into_iter().next() {
                Some((u, f))
            } else {
                None
            }
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let (user, folders) = match user {
        Some((u, f)) => (u, f.into_iter().map(|mf| mf.id).collect()),
        None => {
            return send_response(
                SubsonicResponse::new_error(70, "User not found".into()),
                &params.f,
            )
        }
    };

    send_response(
        SubsonicResponse::new_ok(SubsonicResponseBody::User(User::from_db(
            user, folders,
        ))),
        &params.f,
    )
}
