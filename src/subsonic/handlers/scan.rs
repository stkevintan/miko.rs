use poem::{handler, web::{Data, Query}, IntoResponse};
use crate::subsonic::models::{SubsonicResponse, SubsonicResponseBody, ScanStatus};
use crate::subsonic::common::{SubsonicParams, send_response};
use crate::scanner::Scanner;
use std::collections::HashMap;
use std::sync::Arc;

#[handler]
pub async fn get_scan_status(
    scanner: Data<&Arc<Scanner>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let scanning = scanner.is_scanning();
    let count = if scanning {
        scanner.scan_count()
    } else {
        scanner.total_count()
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ScanStatus(ScanStatus {
        scanning,
        count: Some(count),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn start_scan(
    scanner: Data<&Arc<Scanner>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let incremental = query.get("inc")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(true); 

    let scanner_ptr = Arc::clone(*scanner);
    tokio::spawn(async move {
        if let Err(e) = scanner_ptr.scan_all(incremental).await {
            log::error!("Scan failed: {:?}", e);
        }
    });

    let count = scanner.total_count();

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ScanStatus(ScanStatus {
        scanning: true,
        count: Some(count),
    }));

    send_response(resp, &params.f)
}
