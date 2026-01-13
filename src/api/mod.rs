pub mod handlers;
pub mod models;
pub mod web;
pub mod auth;

use poem::{Route, post, EndpointExt};
use auth::AuthMiddleware;

pub fn create_route() -> Route {
    let auth_routes = Route::new()
        .at("/me", handlers::auth::get_me)
        .at("/scan", post(handlers::scan::start_scan).get(handlers::scan::get_scan_status))
        .at("/stats", handlers::system::get_stats)
        .at("/system", handlers::system::get_system_info)
        .at("/folders", handlers::system::get_folders)
        .at("/now-playing", handlers::system::get_now_playing)
        .at("/coverart/:id", handlers::media::get_cover_art)
        .with(AuthMiddleware);

    Route::new()
        .at("/login", post(handlers::auth::login))
        .nest("/", auth_routes)
}
