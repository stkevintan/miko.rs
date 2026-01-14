use crate::api::auth::verify_jwt;
use crate::config::Config;
use crate::models::user;
use crate::subsonic::auth::{verify_password, verify_token};
use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::SubsonicResponse;
use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

pub struct SubsonicAuth;

impl<E: Endpoint> Middleware<E> for SubsonicAuth {
    type Output = SubsonicAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        SubsonicAuthEndpoint { ep }
    }
}

pub struct SubsonicAuthEndpoint<E> {
    ep: E,
}

impl<E: Endpoint> Endpoint for SubsonicAuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        let mut authenticated_user = None;

        let query =
            serde_urlencoded::from_str::<SubsonicParams>(req.uri().query().unwrap_or_default())
                .unwrap_or_default();

        // 1. Try JWT Auth first (easy for Web UI)
        if let Ok(user) = verify_jwt(&req).await {
            authenticated_user = Some(user);
        } else {
            // 2. Fallback to Subsonic Auth (u/p or u/t/s)
            let db = req.data::<DatabaseConnection>().unwrap();
            let config = req.data::<Arc<Config>>().unwrap();
            let username = match &query.u {
                Some(u) => u,
                None => {
                    let resp = SubsonicResponse::new_error(10, "User not found".to_string());
                    return Ok(send_response(resp, &query.f));
                }
            };

            let user = user::Entity::find()
                .filter(user::Column::Username.eq(username))
                .one(db)
                .await
                .map_err(poem::error::InternalServerError)?;

            if let Some(user) = user {
                let secret_bytes = config.server.password_secret.as_bytes();
                if let Some(password) = &query.p {
                    if verify_password(&user.password, password, secret_bytes) {
                        authenticated_user = Some(user);
                    }
                } else if let (Some(token), Some(salt)) = (&query.t, &query.s) {
                    if verify_token(&user.password, token, salt, secret_bytes) {
                        authenticated_user = Some(user);
                    }
                }
            }
        }

        let user = match authenticated_user {
            Some(u) => u,
            None => {
                let resp =
                    SubsonicResponse::new_error(40, "Wrong username or password".to_string());
                return Ok(send_response(resp, &query.f));
            }
        };

        let username = user.username.clone();
        let client = &query.c.as_deref().unwrap_or("unknown");
        log::debug!(
            "User '{}' authenticated successfully from client '{}'",
            username,
            client
        );

        req.set_data(Arc::new(user));

        self.ep.call(req).await.map(IntoResponse::into_response)
    }
}
