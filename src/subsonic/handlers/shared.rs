use crate::subsonic::common::{send_response, SubsonicParams};
use crate::subsonic::models::SubsonicResponse;
use poem::{handler, web::Query, IntoResponse};

#[handler]
pub async fn not_supported(params: Query<SubsonicParams>) -> impl IntoResponse {
    send_response(
        SubsonicResponse::new_error(0, "Not supported".into()),
        &params.f,
    )
}
