use poem::{handler, http::StatusCode, IntoResponse, web::Path, Body, Response, Route, get};
use rust_embed::RustEmbed;
use mime_guess::from_path;

#[derive(RustEmbed)]
#[folder = "dist/"]
pub struct Asset;

#[handler]
pub async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Asset::get(&path) {
        Some(content) => {
            let mime = from_path(&path).first_or_octet_stream();
            Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(Body::from(content.data.into_owned()))
        }
        None => {
            // Fallback to index.html for SPA routing
            match Asset::get("index.html") {
                Some(index) => Response::builder()
                    .header("Content-Type", "text/html")
                    .body(Body::from(index.data.into_owned())),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty()),
            }
        }
    }
}

#[handler]
pub async fn index_handler() -> impl IntoResponse {
    match Asset::get("index.html") {
        Some(index) => Response::builder()
            .header("Content-Type", "text/html")
            .body(Body::from(index.data.into_owned())),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty()),
    }
}

pub fn create_route() -> Route {
    Route::new()
        .at("/", get(index_handler))
        .at("/*path", get(static_handler))
}
