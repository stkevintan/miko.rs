use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::{
    License, OpenSubsonicExtension, OpenSubsonicExtensions, SubsonicResponse, SubsonicResponseBody,
};
use poem::{handler, web::Query, IntoResponse};

#[handler]
pub async fn ping(params: Query<SubsonicParams>) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::None);

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_license(params: Query<SubsonicParams>) -> impl IntoResponse {
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

#[handler]
pub async fn get_open_subsonic_extensions(params: Query<SubsonicParams>) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::OpenSubsonicExtensions(
        OpenSubsonicExtensions {
            extension: vec![OpenSubsonicExtension {
                name: "songLyrics".to_string(),
                versions: "1".to_string(),
            }],
        },
    ));

    send_response(resp, &params.f)
}
