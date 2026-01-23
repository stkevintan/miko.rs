pub mod auth;
pub mod handlers;
pub mod models;
pub mod web;

use poem::{get, post, EndpointExt, Route};

pub fn create_route(subsonic_routes: Option<Route>) -> Route {
    let mut auth_routes: Route = Route::new()
        .at("/stats", get(handlers::system::get_stats))
        .at("/system", get(handlers::system::get_system_info))
        .at("/folders", get(handlers::system::get_folders).post(handlers::system::create_folder))
        .at("/folders/:id", post(handlers::system::update_folder).delete(handlers::system::delete_folder))
        .at("/songs/:id/tags", get(handlers::library::get_song_tags).post(handlers::library::update_song_tags))
        .at("/songs/:id/scrape-search", get(handlers::library::scrape_search))
        .at("/songs/:id/scrape-tags", get(handlers::library::scrape_song_tags))
        .at("/songs/:id/cover", post(handlers::library::update_song_cover))
        .at("/profile", post(handlers::user::update_profile));

    if let Some(subsonic_routes) = subsonic_routes {
        auth_routes = auth_routes.nest("/", subsonic_routes);
    }

    let auth_routes = auth_routes.with(auth::AuthMiddleware);

    Route::new()
        .at("/login", post(handlers::auth::login))
        .nest("/", auth_routes)
}
