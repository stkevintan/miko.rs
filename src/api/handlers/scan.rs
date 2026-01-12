use poem::{handler, web::{Data, Json}, IntoResponse, http::StatusCode};
use crate::scanner::Scanner;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct ScanQuery {
    pub full: Option<bool>,
}

#[handler]
pub async fn start_scan(
    scanner: Data<&Arc<Scanner>>,
    query: poem::web::Query<ScanQuery>,
) -> impl IntoResponse {
    let full_scan = query.full.unwrap_or(false);
    let scanner_ptr = Arc::clone(*scanner);

    tokio::spawn(async move {
        if let Err(e) = scanner_ptr.scan_all(!full_scan).await {
            log::error!("API Scan failed: {:?}", e);
        }
    });

    StatusCode::ACCEPTED
}

#[handler]
pub async fn get_scan_status(
    scanner: Data<&Arc<Scanner>>,
) -> impl IntoResponse {
    Json(serde_json::json!({
        "scanning": scanner.is_scanning(),
        "count": scanner.scan_count(),
        "total": scanner.total_count(),
    }))
}
