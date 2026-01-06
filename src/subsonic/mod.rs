#[macro_use]
pub mod common;
pub mod models;
pub mod handlers;
pub mod auth;
pub mod middleware;

use poem::{Route, EndpointExt, get};

macro_rules! subsonic_routes {
    ($route:expr, $(($path:literal, $handler:expr)),* $(,)?) => {
        $route
            $(
                .at($path, get($handler))
                .at(concat!($path, ".view"), get($handler))
            )*
    };
}

use crate::subsonic::handlers::{system, browsing, scan};
pub fn create_route() -> Route {
    let route = subsonic_routes!(
        Route::new(),
        // system
        ("/ping", system::ping),
        ("/getLicense", system::get_license),
        ("/getOpenSubsonicExtensions", system::get_open_subsonic_extensions),
        // browsing
        ("/getMusicFolders", browsing::get_music_folders),
        ("/getIndexes", browsing::get_indexes),
        ("/getMusicDirectory", browsing::get_music_directory),
        ("/getGenres", browsing::get_genres),
        ("/getArtists", browsing::get_artists),
        ("/getArtist", browsing::get_artist),
        ("/getAlbum", browsing::get_album),
        ("/getSong", browsing::get_song),
        // scan
        ("/getScanStatus", scan::get_scan_status),
        ("/startScan", scan::start_scan),
    );
    Route::new().nest("/", route.with(middleware::SubsonicAuth))
}
