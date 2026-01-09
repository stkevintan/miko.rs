use poem::{handler, web::{Data, Query}, IntoResponse};
use crate::subsonic::models::{SubsonicResponse, SubsonicResponseBody, ScanStatus};
use crate::subsonic::common::{SubsonicParams, send_response};
use crate::scanner::Scanner;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanQuery {
    #[serde(default)]
    pub full_scan: Option<bool>,
}

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
    query: Query<StartScanQuery>,
) -> impl IntoResponse {
    let incremental = !query.full_scan.unwrap_or(false);

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
