pub mod handlers;
pub mod models;
pub mod web;
pub mod auth;

use poem::{Route, post, EndpointExt};
use auth::AuthMiddleware;

pub fn create_route() -> Route {
    let auth_routes = Route::new()
        .at("/scan", post(handlers::scan::start_scan).get(handlers::scan::get_scan_status))
        .at("/dashboard", post(handlers::system::get_dashboard_data).get(handlers::system::get_dashboard_data))
        .with(AuthMiddleware);

    Route::new()
        .at("/login", post(handlers::auth::login))
        .nest("/", auth_routes)
}
