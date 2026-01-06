use poem::{handler, web::{Data, Json}, IntoResponse, http::StatusCode};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::models::user;
use crate::api::models::{LoginRequest, LoginResponse, Claims, ErrorResponse};
use crate::subsonic::auth::verify_password;
use std::env;
use chrono::Utc;

#[handler]
pub async fn login(
    db: Data<&DatabaseConnection>,
    req: Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match user::Entity::find()
        .filter(user::Column::Username.eq(&req.username))
        .one(*db)
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

    let password_secret = match env::var("PASSWORD_SECRET") {
        Ok(s) => s,
        Err(_) => return Json(ErrorResponse { error: "PASSWORD_SECRET not set".into() })
            .with_status(StatusCode::SERVICE_UNAVAILABLE)
            .into_response(),
    };

    if !verify_password(&user.password, &req.password, password_secret.as_bytes()) {
        return Json(ErrorResponse { error: "Invalid username or password".into() })
            .with_status(StatusCode::UNAUTHORIZED)
            .into_response();
    }

    let jwt_secret = match env::var("JWT_SECRET") {
        Ok(s) => s,
        Err(_) => return Json(ErrorResponse { error: "JWT_SECRET not set".into() })
            .with_status(StatusCode::SERVICE_UNAVAILABLE)
            .into_response(),
    };

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
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => return Json(ErrorResponse { error: "Failed to generate token".into() })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    };

    Json(LoginResponse { token }).into_response()
}
