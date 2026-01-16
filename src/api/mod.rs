pub mod auth;
pub mod handlers;
pub mod models;
pub mod web;

use crate::subsonic::common::SubsonicParams;
use poem::{post, EndpointExt, Route};

pub fn create_route(subsonic_routes: Option<Route>) -> Route {
    let mut auth_routes: Route = Route::new()
        .at("/stats", handlers::system::get_stats)
        .at("/system", handlers::system::get_system_info)
        .at("/folders", handlers::system::get_folders);

    if let Some(subsonic_routes) = subsonic_routes {
        auth_routes = auth_routes.nest("/", subsonic_routes.with(crate::subsonic::middleware::SubsonicParamsMiddleware));
    }

    let auth_routes = auth_routes.with(auth::AuthMiddleware);

    Route::new()
        .at("/login", post(handlers::auth::login))
        .nest("/", auth_routes)
}
