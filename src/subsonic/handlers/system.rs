use actix_web::{web, Responder};
use crate::subsonic::models::{SubsonicResponse, License, OpenSubsonicExtensions, OpenSubsonicExtension};
use crate::subsonic::handlers::{SubsonicParams, send_response};

pub async fn ping(params: web::Query<SubsonicParams>) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let mut resp = SubsonicResponse::new_ok(version);
    resp.ping = Some(());
    
    send_response(resp, &params.f)
}

pub async fn get_license(params: web::Query<SubsonicParams>) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let mut resp = SubsonicResponse::new_ok(version);
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::try_days(365 * 10).unwrap_or_default();
    
    resp.license = Some(License {
        valid: true,
        email: Some("miko@example.com".to_string()),
        license_expires: Some(expires),
        trial_expires: None,
    });
    
    send_response(resp, &params.f)
}

pub async fn get_open_subsonic_extensions(params: web::Query<SubsonicParams>) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let mut resp = SubsonicResponse::new_ok(version);
    resp.open_subsonic_extensions = Some(OpenSubsonicExtensions {
        extension: vec![OpenSubsonicExtension {
            name: "songLyrics".to_string(),
            versions: "1".to_string(),
        }],
    });
    
    send_response(resp, &params.f)
}
