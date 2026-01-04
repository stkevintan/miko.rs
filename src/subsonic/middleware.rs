use actix_web::{
    body::{MessageBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    web,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use crate::subsonic::handlers::{SubsonicParams, send_response};
use crate::subsonic::models::SubsonicResponse;
use crate::subsonic::auth::{verify_password, verify_token};
use crate::models::user;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};

pub struct SubsonicAuth;

impl<S, B> Transform<S, ServiceRequest> for SubsonicAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = SubsonicAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SubsonicAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SubsonicAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SubsonicAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let query = web::Query::<SubsonicParams>::from_query(req.query_string()).unwrap_or_else(|_| web::Query(SubsonicParams {
                u: None, p: None, t: None, s: None, v: None, c: None, f: None
            }));

            let username = match &query.u {
                Some(u) => u,
                None => {
                    let resp = SubsonicResponse::new_error(10, "User not found".to_string(), query.v.clone().unwrap_or_default());
                    let http_resp = send_response(resp, &query.f).map_into_right_body();
                    return Ok(req.into_response(http_resp));
                }
            };

            let db = req.app_data::<web::Data<DatabaseConnection>>().unwrap();
            let user = user::Entity::find()
                .filter(user::Column::Username.eq(username))
                .one(db.get_ref())
                .await
                .map_err(actix_web::error::ErrorInternalServerError)?;

            let user = match user {
                Some(u) => u,
                None => {
                    let resp = SubsonicResponse::new_error(10, "User not found".to_string(), query.v.clone().unwrap_or_default());
                    let http_resp = send_response(resp, &query.f).map_into_right_body();
                    return Ok(req.into_response(http_resp));
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
                let resp = SubsonicResponse::new_error(40, "Wrong username or password".to_string(), query.v.clone().unwrap_or_default());
                let http_resp = send_response(resp, &query.f).map_into_right_body();
                return Ok(req.into_response(http_resp));
            }

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
