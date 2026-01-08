use crate::browser::{Browser, SearchOptions};
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumID3, Artist, ArtistID3, Child, SearchResult, SearchResult2, SearchResult3,
        SubsonicResponse, SubsonicResponseBody,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use std::collections::HashMap;
use std::sync::Arc;

#[handler]
pub async fn search3(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let q = query.get("query").cloned().unwrap_or_default();
    let opts = SearchOptions {
        query: q,
        artist_count: query
            .get("artistCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        artist_offset: query
            .get("artistOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        album_count: query
            .get("albumCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        album_offset: query
            .get("albumOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        song_count: query
            .get("songCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        song_offset: query
            .get("songOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        music_folder_id: query.get("musicFolderId").and_then(|v| v.parse().ok()),
    };

    match browser.search(opts).await {
        Ok((artists, albums, songs)) => {
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::SearchResult3(SearchResult3 {
                    artist: artists.into_iter().map(ArtistID3::from).collect(),
                    album: albums.into_iter().map(AlbumID3::from).collect(),
                    song: songs.into_iter().map(Child::from).collect(),
                }));
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Search error: {:?}", e);
            send_response(
                SubsonicResponse::new_error(0, "Search failed".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn search2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let q = query.get("query").cloned().unwrap_or_default();
    let opts = SearchOptions {
        query: q,
        artist_count: query
            .get("artistCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        artist_offset: query
            .get("artistOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        album_count: query
            .get("albumCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        album_offset: query
            .get("albumOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        song_count: query
            .get("songCount")
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        song_offset: query
            .get("songOffset")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        music_folder_id: query.get("musicFolderId").and_then(|v| v.parse().ok()),
    };

    match browser.search(opts).await {
        Ok((artists, albums, songs)) => {
            // Search2 uses Artist and Child (for albums)
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::SearchResult2(SearchResult2 {
                    artist: artists.into_iter().map(Artist::from).collect(),
                    album: albums
                        .into_iter()
                        .map(|a| {
                            let mut c = Child::from_album_stats(a);
                            c.is_dir = true;
                            c
                        })
                        .collect(),
                    song: songs.into_iter().map(Child::from).collect(),
                }));
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Search error: {:?}", e);
            send_response(
                SubsonicResponse::new_error(0, "Search failed".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn search(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let q = query.get("query").cloned().unwrap_or_default();
    let count = query
        .get("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let offset = query
        .get("offset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    match browser.search_songs(&q, count, offset).await {
        Ok((songs, total_hits)) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SearchResult(SearchResult {
                offset,
                total_hits,
                match_vec: songs.into_iter().map(Child::from).collect(),
            }));
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Search error: {:?}", e);
            send_response(
                SubsonicResponse::new_error(0, "Search failed".into()),
                &params.f,
            )
        }
    }
}
