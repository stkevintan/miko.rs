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

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let query =
            serde_urlencoded::from_str::<SubsonicParams>(req.uri().query().unwrap_or_default())
                .unwrap_or_default();

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

        // Get secret from config
        let config = req.data::<Arc<Config>>().unwrap();
        let secret_bytes = config.server.password_secret.as_bytes();

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

        let mut req = req;
        req.set_data(Arc::new(user));

        self.ep.call(req).await.map(IntoResponse::into_response)
    }
}

// #[derive(Debug, Clone, Copy)]
// pub enum Role {
//     Admin,
//     Settings,
//     Download,
//     Upload,
//     Playlist,
//     CoverArt,
//     Comment,
//     Podcast,
//     Stream,
//     Jukebox,
//     Share,
//     VideoConversion,
// }

// pub struct RequireRole(pub Role);

// impl<E: Endpoint> Middleware<E> for RequireRole {
//     type Output = RequireRoleEndpoint<E>;

//     fn transform(&self, ep: E) -> Self::Output {
//         RequireRoleEndpoint { ep, role: self.0 }
//     }
// }

// pub struct RequireRoleEndpoint<E> {
//     ep: E,
//     role: Role,
// }

// impl<E: Endpoint> Endpoint for RequireRoleEndpoint<E> {
//     type Output = Response;

//     async fn call(&self, req: Request) -> Result<Self::Output> {
//         let query = serde_urlencoded::from_str::<HashMap<String, String>>(
//             req.uri().query().unwrap_or_default(),
//         )
//         .unwrap_or_default();
//         let user = req.data::<user::Model>();
//         let authorized = if let Some(user) = user {
//             user.admin_role
//                 || match self.role {
//                     Role::Admin => user.admin_role,
//                     Role::Settings => user.settings_role,
//                     Role::Download => user.download_role,
//                     Role::Upload => user.upload_role,
//                     Role::Playlist => user.playlist_role,
//                     Role::CoverArt => user.cover_art_role,
//                     Role::Comment => user.comment_role,
//                     Role::Podcast => user.podcast_role,
//                     Role::Stream => user.stream_role,
//                     Role::Jukebox => user.jukebox_role,
//                     Role::Share => user.share_role,
//                     Role::VideoConversion => user.video_conversion_role,
//                 }
//         } else {
//             false
//         };

//         if authorized {
//             return self.ep.call(req).await.map(IntoResponse::into_response);
//         }

//         let resp = SubsonicResponse::new_error(
//             50,
//             "The user is not authorized for the given operation".to_string(),
//         );
//         Ok(send_response(resp, &query.get("f").cloned()))
//     }
// }
