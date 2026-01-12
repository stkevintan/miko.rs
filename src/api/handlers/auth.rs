use poem::{handler, web::{Data, Json}, IntoResponse, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::models::user;
use crate::api::models::{LoginRequest, LoginResponse, Claims, ErrorResponse};
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
