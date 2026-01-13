use crate::api::models::{Claims, CurrentUserResponse, ErrorResponse, LoginRequest, LoginResponse};
use crate::config::Config;
use crate::models::user;
use crate::subsonic::auth::verify_password;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
    IntoResponse,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

#[handler]
pub async fn login(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    req: Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match user::Entity::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(*db) // *db is &DatabaseConnection
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Json(ErrorResponse {
                error: "Invalid username or password".into(),
            })
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response()
        }
        Err(_) => {
            return Json(ErrorResponse {
                error: "Database error".into(),
            })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
        }
    };

    if !verify_password(
        &user.password,
        &req.password,
        config.server.password_secret.as_bytes(),
    ) {
        return Json(ErrorResponse {
            error: "Invalid username or password".into(),
        })
        .with_status(StatusCode::UNAUTHORIZED)
        .into_response();
    }

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::try_days(24).unwrap())
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.username,
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.server.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return Json(ErrorResponse {
                error: "Failed to generate token".into(),
            })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
        }
    };

    Json(LoginResponse { token }).into_response()
}

#[handler]
pub async fn get_me(user: Data<&user::Model>) -> Json<CurrentUserResponse> {
    let roles = [
        (user.admin_role, "admin"),
        (user.settings_role, "settings"),
        (user.download_role, "download"),
        (user.upload_role, "upload"),
        (user.playlist_role, "playlist"),
        (user.cover_art_role, "coverart"),
        (user.comment_role, "comment"),
        (user.podcast_role, "podcast"),
        (user.stream_role, "stream"),
        (user.jukebox_role, "jukebox"),
        (user.share_role, "share"),
        (user.video_conversion_role, "video"),
    ]
    .into_iter()
    .filter_map(|(has_role, role_name)| has_role.then_some(role_name.to_string()))
    .collect::<Vec<_>>();

    Json(CurrentUserResponse {
        username: user.username.clone(),
        email: user.email.clone(),
        roles,
    })
}
