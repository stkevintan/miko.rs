pub mod handlers;
pub mod models;
pub mod web;
pub mod auth;

use poem::{Route, post, EndpointExt};
use auth::AuthMiddleware;

pub fn create_route() -> Route {
    let auth_routes = Route::new()
        .at("/stats", handlers::system::get_stats)
        .at("/system", handlers::system::get_system_info)
        .at("/folders", handlers::system::get_folders)
        .with(AuthMiddleware);

    Route::new()
        .at("/login", post(handlers::auth::login))
        .nest("/", auth_routes)
}
