use crate::browser::{
    map_album_to_id3, map_artist_to_subsonic, map_artist_with_stats_to_id3, map_child_to_subsonic,
    map_genre_to_subsonic, Browser,
};
use crate::config::Config;
use crate::models::music_folder;
use crate::scanner::Scanner;
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumInfo, AlbumWithSongsID3, ArtistInfo, ArtistInfo2, ArtistWithAlbumsID3, ArtistsID3,
        Directory, Genres, Index, IndexID3, Indexes, MusicFolder, MusicFolders, SimilarSongs,
        SimilarSongs2, SubsonicResponse, SubsonicResponseBody, TopSongs,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::collections::HashMap;
use std::sync::Arc;

#[handler]
pub async fn get_music_folders(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let folders = match music_folder::Entity::find().all(*db).await {
        Ok(f) => f,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch music folders".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::MusicFolders(MusicFolders {
        music_folder: folders
            .into_iter()
            .map(|f| MusicFolder {
                id: f.id,
                name: f.name,
            })
            .collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_indexes(
    browser: Data<&Arc<Browser>>,
    config: Data<&Arc<Config>>,
    scanner: Data<&Arc<Scanner>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let music_folder_id = query
        .get("musicFolderId")
        .and_then(|id| id.parse::<i32>().ok());

    match browser
        .get_indexes(music_folder_id, &config.subsonic.ignored_articles)
        .await
    {
        Ok(indexes) => {
            let indexes_vec: Vec<Index> = indexes
                .into_iter()
                .map(|(name, artists)| Index {
                    name,
                    artist: artists.into_iter().map(map_artist_to_subsonic).collect(),
                })
                .collect();

            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Indexes(Indexes {
                last_modified: scanner.last_scan_time() * 1000,
                ignored_articles: config.subsonic.ignored_articles.clone(),
                shortcut: vec![],
                index: indexes_vec,
                child: vec![],
            }));

            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to query indexes: {:?}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to query indexes".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_music_directory(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);
    let offset = query
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);
    let count = query.get("count").and_then(|c| c.parse().ok()).unwrap_or(0);

    match browser.get_directory(&id, offset, count).await {
        Ok(data) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Directory(Directory {
                id: data.dir.id,
                parent: if data.dir.parent.is_empty() {
                    None
                } else {
                    Some(data.dir.parent)
                },
                name: data.dir.title,
                starred: data.dir.starred,
                user_rating: Some(data.dir.user_rating),
                average_rating: Some(data.dir.average_rating),
                play_count: Some(data.dir.play_count),
                total_count: Some(data.total_count),
                child: data
                    .children
                    .into_iter()
                    .map(map_child_to_subsonic)
                    .collect(),
                parents: data
                    .parents
                    .into_iter()
                    .map(map_child_to_subsonic)
                    .collect(),
            }));

            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get directory {}: {:?}", id, e);
            send_response(
                SubsonicResponse::new_error(70, "Directory not found".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_genres(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let genres = match browser.get_genres().await {
        Ok(g) => g,
        Err(e) => {
            log::error!("Failed to query genres: {:?}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to query genres".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Genres(Genres {
        genre: genres.into_iter().map(map_genre_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artists(
    browser: Data<&Arc<Browser>>,
    config: Data<&Arc<Config>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    match browser.get_artists(&config.subsonic.ignored_articles).await {
        Ok(indexes) => {
            let index_vec: Vec<IndexID3> = indexes
                .into_iter()
                .map(|(name, artists)| IndexID3 {
                    name,
                    artist: artists
                        .into_iter()
                        .map(map_artist_with_stats_to_id3)
                        .collect(),
                })
                .collect();

            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Artists(ArtistsID3 {
                ignored_articles: config.subsonic.ignored_articles.clone(),
                index: index_vec,
            }));

            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to query artists: {:?}", e);
            send_response(
                SubsonicResponse::new_error(0, "Failed to query artists".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_artist(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    match browser.get_artist(&id).await {
        Ok((artist, albums)) => {
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::Artist(ArtistWithAlbumsID3 {
                    artist: map_artist_with_stats_to_id3(artist),
                    album: albums.into_iter().map(map_album_to_id3).collect(),
                }));

            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get artist {}: {:?}", id, e);
            send_response(
                SubsonicResponse::new_error(70, "Artist not found".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_album(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    match browser.get_album(&id).await {
        Ok((album, songs)) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Album(AlbumWithSongsID3 {
                album: map_album_to_id3(album),
                song: songs.into_iter().map(map_child_to_subsonic).collect(),
            }));

            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get album {}: {:?}", id, e);
            send_response(
                SubsonicResponse::new_error(70, "Album not found".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_song(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    match browser.get_song(&id).await {
        Ok(song) => {
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::Song(map_child_to_subsonic(song)));
            send_response(resp, &params.f)
        }
        Err(e) => {
            log::error!("Failed to get song {}: {:?}", id, e);
            send_response(
                SubsonicResponse::new_error(70, "Song not found".into()),
                &params.f,
            )
        }
    }
}

#[handler]
pub async fn get_videos(params: Query<SubsonicParams>) -> impl IntoResponse {
    send_response(
        SubsonicResponse::new_error(0, "Not supported".into()),
        &params.f,
    )
}

#[handler]
pub async fn get_video_info(params: Query<SubsonicParams>) -> impl IntoResponse {
    send_response(
        SubsonicResponse::new_error(0, "Not supported".into()),
        &params.f,
    )
}

#[handler]
pub async fn get_artist_info(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ArtistInfo(ArtistInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artist_info2(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ArtistInfo2(ArtistInfo2::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_info(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumInfo(AlbumInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_info2(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumInfo(AlbumInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_similar_songs(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SimilarSongs(SimilarSongs::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_similar_songs2(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("id") {
        return send_response(
            SubsonicResponse::new_error(10, "ID is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SimilarSongs2(SimilarSongs2::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_top_songs(
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if !query.contains_key("artist") {
        return send_response(
            SubsonicResponse::new_error(10, "Artist is required".into()),
            &params.f,
        );
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::TopSongs(TopSongs::default()));
    send_response(resp, &params.f)
}
