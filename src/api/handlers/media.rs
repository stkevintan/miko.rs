use crate::config::Config;
use crate::scanner::utils::get_cover_cache_dir;
use poem::{
    handler,
    http::StatusCode,
    web::{Data, Path, StaticFileRequest},
    IntoResponse,
};
use std::sync::Arc;

#[handler]
pub async fn get_cover_art(
    config: Data<&Arc<Config>>,
    Path(id): Path<String>,
    file_req: StaticFileRequest,
) -> impl IntoResponse {
    let cache_dir = get_cover_cache_dir(&config);
    // Sanitize to prevent path traversal
    let safe_cover_art = std::path::Path::new(&id)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    if safe_cover_art.is_empty() || safe_cover_art != id {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let cache_path = cache_dir.join(safe_cover_art);

    if cache_path.exists() {
        return match file_req.create_response(&cache_path, false, false) {
            Ok(resp) => resp.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };
    }

    StatusCode::NOT_FOUND.into_response()
}
