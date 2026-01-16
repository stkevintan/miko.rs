use crate::api::models::{ErrorResponse, UpdateProfileRequest};
use crate::config::Config;
use crate::crypto::encrypt;
use crate::models::user;
use crate::subsonic::auth::verify_password;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Json},
    IntoResponse,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel, Set};
use std::sync::Arc;

#[handler]
pub async fn update_profile(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    user: Data<&Arc<user::Model>>,
    req: Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    // 1. Verify current password for ANY change
    if !verify_password(
        &user.password,
        &req.current_password,
        config.server.password_secret.as_bytes(),
    ) {
        return Json(ErrorResponse {
            error: "Invalid current password".into(),
        })
        .with_status(StatusCode::UNAUTHORIZED)
        .into_response();
    }

    let mut user_active: user::ActiveModel = user.as_ref().clone().into_active_model();
    let mut changed = false;

    // 2. Update email if provided
    if user.email != req.email {
        user_active.email = Set(req.email.clone());
        changed = true;
    }

    // 3. Update password if new_password is not blank
    if let Some(new_pwd) = &req.new_password {
        if !new_pwd.trim().is_empty() {
            let encrypted_password = match encrypt(new_pwd, config.server.password_secret.as_bytes()) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to encrypt password for user '{}': {}", user.username, e);
                    return Json(ErrorResponse {
                        error: "Encryption error".into(),
                    })
                    .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response();
                }
            };
            user_active.password = Set(encrypted_password);
            changed = true;
        }
    }

    if !changed {
        return StatusCode::OK.into_response();
    }

    user_active.updated_at = Set(chrono::Utc::now());

    match user_active.update(*db).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => {
            log::error!("Failed to update profile for user '{}': {}", user.username, e);
            Json(ErrorResponse {
                error: "Database error".into(),
            })
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response()
        }
    }
}
