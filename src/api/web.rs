use mime_guess::from_path;
use poem::{get, handler, http::StatusCode, web::Path, Body, IntoResponse, Response, Route};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "dist/"]
pub struct Asset;

fn serve_index() -> Response {
    match Asset::get("index.html") {
        Some(index) => Response::builder()
            .header("Content-Type", "text/html")
            .body(Body::from(index.data.into_owned())),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty()),
    }
}

#[handler]
pub async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Asset::get(&path) {
        Some(content) => {
            let mime = from_path(&path).first_or_octet_stream();
            Response::builder()
                .header("Content-Type", mime.as_ref())
                .body(Body::from(content.data.into_owned()))
        }
        None => serve_index(),
    }
}

#[handler]
pub async fn index_handler() -> impl IntoResponse {
    serve_index()
}

pub fn create_route() -> Route {
    Route::new()
        .at("/", get(index_handler))
        .at("/*path", get(static_handler))
}
