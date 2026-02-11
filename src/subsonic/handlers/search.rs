use crate::service::search::SearchOptions;
use crate::service::Service;
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
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Search23Query {
    pub query: String,
    pub artist_count: Option<u64>,
    pub artist_offset: Option<u64>,
    pub album_count: Option<u64>,
    pub album_offset: Option<u64>,
    pub song_count: Option<u64>,
    pub song_offset: Option<u64>,
    pub music_folder_id: Option<i32>,
}

impl From<Search23Query> for SearchOptions {
    fn from(q: Search23Query) -> Self {
        Self {
            query: q.query,
            artist_count: q.artist_count.unwrap_or(20),
            artist_offset: q.artist_offset.unwrap_or(0),
            album_count: q.album_count.unwrap_or(20),
            album_offset: q.album_offset.unwrap_or(0),
            song_count: q.song_count.unwrap_or(20),
            song_offset: q.song_offset.unwrap_or(0),
            music_folder_id: q.music_folder_id,
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub count: Option<u64>,
    pub offset: Option<u64>,
}

#[handler]
pub async fn search3(
    service: Data<&Arc<Service>>,
    params: Data<&SubsonicParams>,
    query: Query<Search23Query>,
) -> impl IntoResponse {
    let opts = SearchOptions::from(query.0);

    match service.search(opts).await {
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
    service: Data<&Arc<Service>>,
    params: Data<&SubsonicParams>,
    query: Query<Search23Query>,
) -> impl IntoResponse {
    let opts = SearchOptions::from(query.0);

    match service.search(opts).await {
        Ok((artists, albums, songs)) => {
            // Search2 uses Artist and Child (for albums)
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::SearchResult2(SearchResult2 {
                    artist: artists.into_iter().map(Artist::from).collect(),
                    album: albums
                        .into_iter()
                        .map(|a| Child::from_album_stats(a))
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
    service: Data<&Arc<Service>>,
    params: Data<&SubsonicParams>,
    query: Query<SearchQuery>,
) -> impl IntoResponse {
    let q = &query.query;
    let count = query.count.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);

    match service.search_songs(q, count, offset).await {
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
