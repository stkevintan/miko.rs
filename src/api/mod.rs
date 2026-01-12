pub mod handlers;
pub mod models;
pub mod web;

use poem::{Route, post};

pub fn create_route() -> Route {
    Route::new()
        .at("/login", post(handlers::login))
}
