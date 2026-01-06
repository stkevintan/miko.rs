pub mod handlers;
pub mod models;

use poem::{Route, post};

pub fn create_route() -> Route {
    Route::new()
        .at("/login", post(handlers::login))
}
