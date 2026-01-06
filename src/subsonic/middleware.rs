use poem::{Middleware, Endpoint, Request, Response, Result, IntoResponse};
use crate::subsonic::common::{SubsonicParams, send_response};
use crate::subsonic::models::SubsonicResponse;
use crate::subsonic::auth::{verify_password, verify_token};
use crate::models::user;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};


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

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let query = serde_urlencoded::from_str::<SubsonicParams>(req.uri().query().unwrap_or_default())
            .unwrap_or(SubsonicParams {
                u: None, p: None, t: None, s: None, c: None, f: None
            });

        let username = match &query.u {
            Some(u) => u,
            None => {
                let resp = SubsonicResponse::new_error(10, "User not found".to_string());
                return Ok(send_response(resp, &query.f));
            }
        };

        let db = req.data::<DatabaseConnection>().unwrap();
        let user = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(poem::error::InternalServerError)?;

        let user = match user {
            Some(u) => u,
            None => {
                let resp = SubsonicResponse::new_error(10, "User not found".to_string());
                return Ok(send_response(resp, &query.f));
            }
        };

        // Get secret from config (hardcoded for now or from env)
        let secret = std::env::var("PASSWORD_SECRET").unwrap_or_default();
        let secret_bytes = secret.as_bytes();

        let mut authenticated = false;
        if let Some(password) = &query.p {
            if verify_password(&user.password, password, secret_bytes) {
                authenticated = true;
            }
        } else if let (Some(token), Some(salt)) = (&query.t, &query.s) {
            if verify_token(&user.password, token, salt, secret_bytes) {
                authenticated = true;
            }
        }

        if !authenticated {
            let resp = SubsonicResponse::new_error(40, "Wrong username or password".to_string());
            return Ok(send_response(resp, &query.f));
        }

        self.ep.call(req).await.map(IntoResponse::into_response)
    }
}
