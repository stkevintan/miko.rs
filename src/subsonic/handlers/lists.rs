use crate::browser::{AlbumListOptions, Browser};
use crate::subsonic::{
    common::{send_response, SubsonicParams},
    models::{
        AlbumID3, AlbumList, AlbumList2, Artist, ArtistID3, Child, NowPlaying, NowPlayingEntry,
        RandomSongs, SongsByGenre, Starred, Starred2, SubsonicResponse, SubsonicResponseBody,
    },
};
use chrono::Utc;
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsByGenreQuery {
    pub genre: String,
    #[serde(default = "default_count")]
    pub count: u64,
    #[serde(default)]
    pub offset: u64,
    pub music_folder_id: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolderQuery {
    pub music_folder_id: Option<i32>,
}

fn default_count() -> u64 {
    10
}

#[handler]
pub async fn get_album_list2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let albums = match browser.get_albums(list_params.0).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch albums".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumList2(AlbumList2 {
        album: albums.into_iter().map(AlbumID3::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album_list(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let albums = match browser.get_albums(list_params.0).await {
        Ok(a) => a,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch albums".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::AlbumList(AlbumList {
        album: albums.into_iter().map(Child::from_album_stats).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_random_songs(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    list_params: Query<AlbumListOptions>,
) -> impl IntoResponse {
    let songs = match browser.get_random_songs(list_params.0).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch songs".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::RandomSongs(RandomSongs {
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_songs_by_genre(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<SongsByGenreQuery>,
) -> impl IntoResponse {
    let songs = match browser
        .get_songs_by_genre(
            &query.genre,
            query.count,
            query.offset,
            query.music_folder_id,
        )
        .await
    {
        Ok(s) => s,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch songs".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::SongsByGenre(SongsByGenre {
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<MusicFolderQuery>,
) -> impl IntoResponse {
    let music_folder_id = query.music_folder_id;

    let (artists, albums, songs) = match browser.get_starred_items(music_folder_id).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch starred items".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Starred(Starred {
        artist: artists.into_iter().map(Artist::from).collect(),
        album: albums.into_iter().map(Child::from_album_stats).collect(),
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_starred2(
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
    query: Query<MusicFolderQuery>,
) -> impl IntoResponse {
    let music_folder_id = query.music_folder_id;

    let (artists, albums, songs) = match browser.get_starred_items(music_folder_id).await {
        Ok(res) => res,
        Err(e) => {
            log::error!("Database error: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch starred items".into()),
                &params.f,
            );
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Starred2(Starred2 {
        artist: artists.into_iter().map(ArtistID3::from).collect(),
        album: albums.into_iter().map(AlbumID3::from).collect(),
        song: songs.into_iter().map(Child::from).collect(),
    }));

    send_response(resp, &params.f)
}

const NOW_PLAYING_EXPIRATION_MINUTES: i64 = 10;

#[handler]
pub async fn get_now_playing(
    db: Data<&DatabaseConnection>,
    browser: Data<&Arc<Browser>>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    use crate::models::now_playing;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    // Clean up outdated records (older than 10 minutes)
    let ten_minutes_ago = Utc::now() - chrono::Duration::minutes(NOW_PLAYING_EXPIRATION_MINUTES);
    let delete_result = now_playing::Entity::delete_many()
        .filter(now_playing::Column::UpdatedAt.lt(ten_minutes_ago))
        .exec(db.0)
        .await;

    if let Err(e) = delete_result {
        log::error!("Failed to clean up now playing records: {}", e);
    }

    // Fetch current now playing records
    let current_now_playing = match now_playing::Entity::find().all(db.0).await {
        Ok(np) => np,
        Err(e) => {
            log::error!("Failed to fetch now playing records: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Internal error".into()),
                &params.f,
            );
        }
    };

    if current_now_playing.is_empty() {
        return send_response(
            SubsonicResponse::new_ok(SubsonicResponseBody::NowPlaying(NowPlaying {
                entry: Vec::new(),
            })),
            &params.f,
        );
    }

    let song_ids: Vec<String> = current_now_playing
        .iter()
        .map(|r| r.song_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let songs = match browser.get_songs_by_ids(&song_ids).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to get songs for now playing: {}", e);
            return send_response(
                SubsonicResponse::new_error(0, "Internal error".into()),
                &params.f,
            );
        }
    };

    let song_map: std::collections::HashMap<String, crate::browser::types::ChildWithMetadata> =
        songs.into_iter().map(|s| (s.id.clone(), s)).collect();

    let now = Utc::now();
    let mut entries = Vec::new();

    for record in current_now_playing {
        if let Some(song_metadata) = song_map.get(&record.song_id) {
            let minutes_ago = (now - record.updated_at).num_minutes() as i32;
            entries.push(NowPlayingEntry {
                child: song_metadata.clone().into(),
                username: record.username,
                minutes_ago,
                player_name: record.player_name,
            });
        }
    }

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::NowPlaying(NowPlaying {
        entry: entries,
    }));

    send_response(resp, &params.f)
}
