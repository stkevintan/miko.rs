use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::SubsonicResponse;
use poem::{handler, web::Data, IntoResponse};

#[handler]
pub async fn not_supported(params: Data<&SubsonicParams>) -> impl IntoResponse {
    send_response(
        SubsonicResponse::new_error(0, "Not supported".into()),
        &params.f,
    )
}

#[handler]
pub async fn not_implemented(params: Data<&SubsonicParams>) -> impl IntoResponse {
    send_response(
        SubsonicResponse::new_error(0, "Not implemented".into()),
        &params.f,
    )
}
