use poem::{
    http::header,
    Endpoint, Middleware, Request, Response, Result, Error,
    http::StatusCode, IntoResponse,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::api::models::Claims;
use crate::models::user;
use crate::config::Config;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
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

impl<E: Endpoint> Endpoint for AuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        let config = req.data::<Arc<Config>>().ok_or_else(|| {
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        let db = req.data::<DatabaseConnection>().ok_or_else(|| {
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }

        let token = &auth_header[7..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.server.jwt_secret.as_bytes()),
            &Validation::default(),
        ).map_err(|_| Error::from_status(StatusCode::UNAUTHORIZED))?;

        let user = user::Entity::find()
            .filter(user::Column::Username.eq(token_data.claims.sub))
            .one(db)
            .await
            .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?
            .ok_or_else(|| Error::from_status(StatusCode::UNAUTHORIZED))?;

        // Insert user into request data
        req.set_data(user);

        let resp = self.ep.call(req).await?;
        Ok(resp.into_response())
    }
}
