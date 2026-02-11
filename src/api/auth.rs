use crate::api::models::Claims;
use crate::config::Config;
use crate::models::user;
use jsonwebtoken::{decode, DecodingKey, Validation};
use poem::{
    http::StatusCode, Endpoint, Error, IntoResponse, Middleware, Request, Response, Result,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

pub struct AuthMiddleware;

impl<E: Endpoint> Middleware<E> for AuthMiddleware {
    type Output = AuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthEndpoint { ep }
    }
}

pub struct AuthEndpoint<E> {
    ep: E,
}

async fn verify_jwt(req: &Request) -> Result<user::Model, Error> {
    let config = req
        .data::<Arc<Config>>()
        .ok_or_else(|| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let db = req
        .data::<DatabaseConnection>()
        .ok_or_else(|| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let token_data = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .and_then(|t| {
            decode::<Claims>(
                t,
                &DecodingKey::from_secret(config.server.jwt_secret.as_bytes()),
                &Validation::default(),
            )
            .ok()
        })
        .ok_or_else(|| {
            log::debug!("JWT decoding failed or missing token");
            Error::from_status(StatusCode::UNAUTHORIZED)
        })?;

    let user = user::Entity::find()
        .filter(user::Column::Username.eq(&token_data.claims.sub))
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Database error during authentication: {}", e);
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?
        .ok_or_else(|| {
            log::debug!("User not found for token: {}", &token_data.claims.sub);
            Error::from_status(StatusCode::UNAUTHORIZED)
        })?;

    Ok(user)
}

impl<E: Endpoint> Endpoint for AuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        let user = verify_jwt(&req).await?;
        // Insert user into request data
        req.set_data(Arc::new(user));

        let resp = self.ep.call(req).await?;
        Ok(resp.into_response())
    }
}
