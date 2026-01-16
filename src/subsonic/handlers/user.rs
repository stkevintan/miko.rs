use crate::config::Config;
use crate::crypto::encrypt;
use crate::models::{user, music_folder};
use crate::subsonic::common::{send_response, SubsonicParams, deserialize_optional_bool};
use crate::subsonic::models::{
    SubsonicResponse, SubsonicResponseBody, User, Users,
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use serde::Deserialize;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, IntoActiveModel};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetUserQuery {
    pub username: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserQuery {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool")]
    pub admin_role: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserQuery {
    pub username: String,
    pub password: Option<String>,
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool")]
    pub admin_role: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeleteUserQuery {
    pub username: String,
}

#[handler]
pub async fn get_users(
    db: Data<&DatabaseConnection>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
) -> impl IntoResponse {
    if !current_user.admin_role {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let users_with_folders = match user::Entity::find()
        .find_with_related(music_folder::Entity)
        .all(*db)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let subsonic_users: Vec<User> = users_with_folders
        .into_iter()
        .map(|(u, f)| User::from_db(u, f.into_iter().map(|mf| mf.id).collect()))
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
    params: Data<&SubsonicParams>,
    query: Query<GetUserQuery>,
) -> impl IntoResponse {
    let username = &query.username;

    if !current_user.admin_role && current_user.username != *username {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let (user, folders) = match user::Entity::find_by_id(username.to_string())
        .find_with_related(music_folder::Entity)
        .all(*db)
        .await
    {
        Ok(mut res) if !res.is_empty() => {
            let (u, f) = res.remove(0);
            (u, f.into_iter().map(|mf| mf.id).collect::<Vec<_>>())
        }
        Ok(_) => {
            return send_response(
                SubsonicResponse::new_error(70, "User not found".into()),
                &params.f,
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    send_response(
        SubsonicResponse::new_ok(SubsonicResponseBody::User(User::from_db(
            user, folders,
        ))),
        &params.f,
    )
}

#[handler]
pub async fn create_user(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<CreateUserQuery>,
) -> impl IntoResponse {
    if !current_user.admin_role {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let encrypted_password = match encrypt(&query.password, config.server.password_secret.as_bytes()) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Encryption error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Encryption error".into()),
                &params.f,
            );
        }
    };

    let user = user::ActiveModel {
        username: Set(query.username.clone()),
        password: Set(encrypted_password),
        email: Set(query.email.clone()),
        admin_role: Set(query.admin_role.unwrap_or(false)),
        // "let the rest role always be true"
        settings_role: Set(true),
        download_role: Set(true),
        upload_role: Set(true),
        playlist_role: Set(true),
        cover_art_role: Set(true),
        comment_role: Set(true),
        podcast_role: Set(true),
        stream_role: Set(true),
        share_role: Set(true),
        scrobbling_enabled: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    match user.insert(*db).await {
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn update_user(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<UpdateUserQuery>,
) -> impl IntoResponse {
    if !current_user.admin_role {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    let user = match user::Entity::find_by_id(query.username.clone()).one(*db).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "User not found".into()),
                &params.f,
            );
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            );
        }
    };

    let mut user_active = user.into_active_model();

    if let Some(password) = &query.password {
        if !password.is_empty() {
            match encrypt(password, config.server.password_secret.as_bytes()) {
                Ok(p) => user_active.password = Set(p),
                Err(e) => {
                    log::error!("Encryption error: {}", e);
                    return send_response(
                        SubsonicResponse::new_error(0, "Encryption error".into()),
                        &params.f,
                    );
                }
            }
        }
    }

    if let Some(email) = &query.email {
        user_active.email = Set(Some(email.clone()));
    }

    if let Some(admin_role) = query.admin_role {
        user_active.admin_role = Set(admin_role);
    }

    user_active.updated_at = Set(chrono::Utc::now());

    match user_active.update(*db).await {
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn delete_user(
    db: Data<&DatabaseConnection>,
    current_user: Data<&Arc<user::Model>>,
    params: Data<&SubsonicParams>,
    query: Query<DeleteUserQuery>,
) -> impl IntoResponse {
    if !current_user.admin_role {
        return send_response(
            SubsonicResponse::new_error(40, "The user is not authorized for the given operation.".into()),
            &params.f,
        );
    }

    if current_user.username == query.username {
        return send_response(
            SubsonicResponse::new_error(0, "Cannot delete yourself".into()),
            &params.f,
        );
    }

    match user::Entity::delete_by_id(query.username.clone())
        .exec(*db)
        .await
    {
        Ok(_) => send_response(SubsonicResponse::new_ok(SubsonicResponseBody::None), &params.f),
        Err(e) => {
            log::error!("Database error: {}", e);
            send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    }
}
