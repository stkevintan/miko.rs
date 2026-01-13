use poem::{handler, web::{Data, Json}, IntoResponse, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::models::user;
use crate::api::models::{LoginRequest, LoginResponse, Claims, ErrorResponse, CurrentUserResponse};
use crate::subsonic::auth::verify_password;
use crate::config::Config;
use chrono::Utc;
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
        Ok(None) => return Json(ErrorResponse { error: "Invalid username or password".into() })
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response(),
        Err(_) => return Json(ErrorResponse { error: "Database error".into() })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    };

    if !verify_password(&user.password, &req.password, config.server.password_secret.as_bytes()) {
        return Json(ErrorResponse { error: "Invalid username or password".into() })
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
        Err(_) => return Json(ErrorResponse { error: "Failed to generate token".into() })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    };

    Json(LoginResponse { token }).into_response()
}

#[handler]
pub async fn get_me(user: Data<&user::Model>) -> Json<CurrentUserResponse> {
    let mut roles = Vec::new();
    if user.admin_role { roles.push("admin".into()); }
    if user.settings_role { roles.push("settings".into()); }
    if user.download_role { roles.push("download".into()); }
    if user.upload_role { roles.push("upload".into()); }
    if user.playlist_role { roles.push("playlist".into()); }
    if user.cover_art_role { roles.push("coverart".into()); }
    if user.comment_role { roles.push("comment".into()); }
    if user.podcast_role { roles.push("podcast".into()); }
    if user.stream_role { roles.push("stream".into()); }
    if user.jukebox_role { roles.push("jukebox".into()); }
    if user.share_role { roles.push("share".into()); }
    if user.video_conversion_role { roles.push("video".into()); }

    Json(CurrentUserResponse {
        username: user.username.clone(),
        email: user.email.clone(),
        roles,
    })
}
