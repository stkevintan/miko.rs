use actix_web::{web, Responder};
use crate::subsonic::models::{SubsonicResponse, SubsonicResponseBody, License, OpenSubsonicExtensions, OpenSubsonicExtension};
use crate::subsonic::handlers::{SubsonicParams, send_response};

pub async fn ping(params: web::Query<SubsonicParams>) -> impl Responder {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::None);
    
    send_response(resp, &params.f)
}

pub async fn get_license(params: web::Query<SubsonicParams>) -> impl Responder {
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::try_days(365 * 10).unwrap_or_default();
    
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::License(License {
        valid: true,
        email: Some("miko@example.com".to_string()),
        license_expires: Some(expires),
        trial_expires: None,
    }));
    
    send_response(resp, &params.f)
}

pub async fn get_open_subsonic_extensions(params: web::Query<SubsonicParams>) -> impl Responder {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::OpenSubsonicExtensions(OpenSubsonicExtensions {
        extension: vec![OpenSubsonicExtension {
            name: "songLyrics".to_string(),
            versions: "1".to_string(),
        }],
    }));
    
    send_response(resp, &params.f)
}
