#[macro_use]
pub mod common;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod models;

use poem::{EndpointExt, Route};

macro_rules! subsonic_routes {
    ($route:expr, $(($path:literal, $handler:expr)),* $(,)?) => {
        $route
            $(
                .at($path, $handler)
                .at(concat!($path, ".view"), $handler)
            )*
    };
}

use crate::subsonic::{
    common::SubsonicParams,
    handlers::{
        annotation, bookmarks, browsing, lists, media, playlists, scan, search, shared, system,
        user,
    },
};

fn base_routes() -> Route {
    subsonic_routes!(
        Route::new(),
        // system
        ("/ping", system::ping),
        ("/getLicense", system::get_license),
        (
            "/getOpenSubsonicExtensions",
            system::get_open_subsonic_extensions
        ),
        // browsing
        ("/getMusicFolders", browsing::get_music_folders),
        ("/getIndexes", browsing::get_indexes),
        ("/getMusicDirectory", browsing::get_music_directory),
        ("/getGenres", browsing::get_genres),
        ("/getArtists", browsing::get_artists),
        ("/getArtist", browsing::get_artist),
        ("/getAlbum", browsing::get_album),
        ("/getSong", browsing::get_song),
        ("/getVideos", shared::not_supported),
        ("/getVideoInfo", shared::not_supported),
        ("/getArtistInfo", browsing::get_artist_info),
        ("/getArtistInfo2", browsing::get_artist_info2),
        ("/getAlbumInfo", browsing::get_album_info),
        ("/getAlbumInfo2", browsing::get_album_info2),
        ("/getSimilarSongs", browsing::get_similar_songs),
        ("/getSimilarSongs2", browsing::get_similar_songs2),
        ("/getTopSongs", browsing::get_top_songs),
        // media retrieval
        ("/stream", media::stream),
        ("/download", media::download),
        ("/hls.m3u8", shared::not_supported),
        ("/getCaptions", shared::not_supported),
        ("/getCoverArt", media::get_cover_art),
        ("/getLyrics", media::get_lyrics),
        ("/getLyricsBySongId", media::get_lyrics_by_song_id),
        ("/getAvatar", media::get_avatar),
        // playlists
        ("/getPlaylists", playlists::get_playlists),
        ("/getPlaylist", playlists::get_playlist),
        ("/createPlaylist", playlists::create_playlist),
        ("/updatePlaylist", playlists::update_playlist),
        ("/deletePlaylist", playlists::delete_playlist),
        // scan
        ("/getScanStatus", scan::get_scan_status),
        ("/startScan", scan::start_scan),
        // search
        ("/search", search::search),
        ("/search2", search::search2),
        ("/search3", search::search3),
        ("/getChatMessages", shared::not_implemented),
        ("/addChatMessage", shared::not_implemented),
        // user management
        ("/getUser", user::get_user),
        ("/getUsers", user::get_users),
        ("/createUser", user::create_user),
        ("/updateUser", user::update_user),
        ("/deleteUser", user::delete_user),
        ("/changePassword", shared::not_supported),
        // list
        ("/getAlbumList", lists::get_album_list),
        ("/getAlbumList2", lists::get_album_list2),
        ("/getRandomSongs", lists::get_random_songs),
        ("/getSongsByGenre", lists::get_songs_by_genre),
        ("/getNowPlaying", lists::get_now_playing),
        ("/getStarred", lists::get_starred),
        ("/getStarred2", lists::get_starred2),
        // annotation
        ("/star", annotation::star),
        ("/unstar", annotation::unstar),
        ("/setRating", annotation::set_rating),
        ("/scrobble", annotation::scrobble),
        // bookmarks
        ("/getBookmarks", bookmarks::get_bookmarks),
        ("/createBookmark", bookmarks::create_bookmark),
        ("/deleteBookmark", bookmarks::delete_bookmark),
        ("/getPlayQueue", bookmarks::get_play_queue),
        ("/savePlayQueue", bookmarks::save_play_queue),
    )
}

pub fn create_api_route() -> Route {
    Route::new().nest("/", base_routes().data(SubsonicParams::default()))
}

pub fn create_route() -> Route {
    Route::new().nest(
        "/",
        base_routes()
            .with(middleware::SubsonicAuth)
            .with(middleware::SubsonicParamsMiddleware),
    )
}
