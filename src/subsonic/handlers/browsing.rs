use crate::service::Service;
use crate::config::Config;
use crate::models::music_folder;
use crate::scanner::Scanner;
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumID3, AlbumInfo, AlbumWithSongsID3, Artist, ArtistID3, ArtistInfo, ArtistInfo2,
        ArtistWithAlbumsID3, ArtistsID3, Child, Directory, Genre, Genres, Index, IndexID3, Indexes,
        MusicFolder, MusicFolders, SimilarSongs, SimilarSongs2, SubsonicResponse,
        SubsonicResponseBody, TopSongs,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIndexesQuery {
    pub music_folder_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct GetMusicDirectoryQuery {
    pub id: String,
    pub offset: Option<u64>,
    pub count: Option<u64>,
}

#[derive(Deserialize)]
pub struct IdQuery {
    pub id: String,
}

#[derive(Deserialize)]
pub struct ArtistQuery {
    pub artist: String,
    pub count: Option<u64>,
}

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
    service: Data<&Arc<Service>>,
    config: Data<&Arc<Config>>,
    scanner: Data<&Arc<Scanner>>,
    params: Query<SubsonicParams>,
    query: Query<GetIndexesQuery>,
) -> impl IntoResponse {
    let music_folder_id = query.music_folder_id;

    match service
        .get_indexes(music_folder_id, &config.subsonic.ignored_articles)
        .await
    {
        Ok(indexes) => {
            let indexes_vec: Vec<Index> = indexes
                .into_iter()
                .map(|(name, artists)| Index {
                    name,
                    artist: artists.into_iter().map(Artist::from).collect(),
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
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
    query: Query<GetMusicDirectoryQuery>,
) -> impl IntoResponse {
    let id = &query.id;
    let offset = query.offset.unwrap_or(0);
    let count = query.count.unwrap_or(0);

    match service.get_directory(id, offset, count).await {
        Ok(data) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Directory(Directory {
                id: data.dir.id,
                parent: data.dir.parent,
                name: data.dir.title,
                starred: data.dir.starred,
                user_rating: Some(data.dir.user_rating),
                average_rating: Some(data.dir.average_rating),
                play_count: Some(data.dir.play_count),
                total_count: Some(data.total_count),
                child: data.children.into_iter().map(Child::from).collect(),
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
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let genres = match service.get_genres().await {
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
        genre: genres.into_iter().map(Genre::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artists(
    service: Data<&Arc<Service>>,
    config: Data<&Arc<Config>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    match service.get_artists(&config.subsonic.ignored_articles).await {
        Ok(indexes) => {
            let index_vec: Vec<IndexID3> = indexes
                .into_iter()
                .map(|(name, artists)| IndexID3 {
                    name,
                    artist: artists.into_iter().map(ArtistID3::from).collect(),
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
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
    query: Query<IdQuery>,
) -> impl IntoResponse {
    let id = &query.id;

    match service.get_artist(id).await {
        Ok((artist, albums)) => {
            let resp =
                SubsonicResponse::new_ok(SubsonicResponseBody::Artist(ArtistWithAlbumsID3 {
                    artist: ArtistID3::from(artist),
                    album: albums.into_iter().map(AlbumID3::from).collect(),
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
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
    query: Query<IdQuery>,
) -> impl IntoResponse {
    let id = &query.id;

    match service.get_album(id).await {
        Ok((album, songs)) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Album(AlbumWithSongsID3 {
                album: AlbumID3::from(album),
                song: songs.into_iter().map(Child::from).collect(),
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
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
    query: Query<IdQuery>,
) -> impl IntoResponse {
    let id = &query.id;

    match service.get_song(id).await {
        Ok(song) => {
            let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Song(Child::from(song)));
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
pub async fn get_artist_info(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ArtistInfo(ArtistInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artist_info2(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::ArtistInfo2(ArtistInfo2::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_info(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumInfo(AlbumInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_info2(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumInfo(AlbumInfo::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_similar_songs(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SimilarSongs(SimilarSongs::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_similar_songs2(
    params: Query<SubsonicParams>,
    _query: Query<IdQuery>,
) -> impl IntoResponse {
    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SimilarSongs2(SimilarSongs2::default()));
    send_response(resp, &params.f)
}

#[handler]
pub async fn get_top_songs(
    service: Data<&Arc<Service>>,
    params: Query<SubsonicParams>,
    query: Query<ArtistQuery>,
) -> impl IntoResponse {
    let count = query.count.unwrap_or(50);
    let songs = match service.get_top_songs(&query.artist, count).await {
        Ok(songs) => songs,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch top songs".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::TopSongs(TopSongs {
        song: songs.into_iter().map(Child::from).collect(),
    }));
    send_response(resp, &params.f)
}
