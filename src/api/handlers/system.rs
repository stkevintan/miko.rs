use poem::{handler, web::{Data, Json}};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, ColumnTrait, QueryFilter, LoaderTrait, QuerySelect};
use crate::models::{child, album, artist, now_playing, song_artist, genre, music_folder};
use serde::Serialize;
use sysinfo::System;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static SYS: Lazy<Mutex<System>> = Lazy::new(|| {
    let mut sys = System::new_all();
    sys.refresh_all();
    Mutex::new(sys)
});

#[derive(Serialize)]
pub struct Stats {
    pub songs: u64,
    pub albums: u64,
    pub artists: u64,
    pub genres: u64,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
}

#[derive(Serialize)]
pub struct FolderInfo {
    pub label: String,
    pub path: String,
    pub song_count: u64,
}

#[derive(Serialize)]
pub struct NowPlayingInfo {
    pub username: String,
    pub player_name: String,
    pub song_title: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub album_id: Option<String>,
    pub cover_art: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[handler]
pub async fn get_stats(
    db: Data<&DatabaseConnection>,
) -> Result<Json<Stats>, poem::Error> {
    let (songs, albums, artists, genres) = tokio::try_join!(
        child::Entity::find().filter(child::Column::IsDir.eq(false)).count(*db),
        album::Entity::find().count(*db),
        artist::Entity::find().count(*db),
        genre::Entity::find().count(*db),
    ).map_err(|e| {
        log::error!("Failed to fetch stats: {}", e);
        poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(Json(Stats { songs, albums, artists, genres }))
}

#[handler]
pub async fn get_now_playing(
    db: Data<&DatabaseConnection>,
) -> Result<Json<Vec<NowPlayingInfo>>, poem::Error> {
    let now_playing_list = now_playing::Entity::find()
        .all(*db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch now playing list: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let song_ids: Vec<String> = now_playing_list.iter().map(|np| np.song_id.clone()).collect();
    
    let songs_with_albums = if !song_ids.is_empty() {
        child::Entity::find()
            .filter(child::Column::Id.is_in(song_ids))
            .find_also_related(album::Entity)
            .all(*db)
            .await
            .map_err(|e| {
                log::error!("Failed to fetch songs with albums for now playing: {}", e);
                poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?
    } else {
        Vec::new()
    };

    let song_models: Vec<child::Model> = songs_with_albums.iter().map(|(s, _)| s.clone()).collect();
    let artists_per_song = if !song_models.is_empty() {
        song_models.load_many_to_many(artist::Entity, song_artist::Entity, *db)
            .await
            .map_err(|e| {
                log::error!("Failed to load artists for now playing songs: {}", e);
                poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?
    } else {
        Vec::new()
    };

    let mut song_map = HashMap::new();
    for (i, (song, album)) in songs_with_albums.into_iter().enumerate() {
        let artists = artists_per_song.get(i).cloned().unwrap_or_default();
        song_map.insert(song.id.clone(), (song, album, artists));
    }

    let mut now_playing_info = Vec::new();
    for np in now_playing_list {
        let entry = song_map.get(&np.song_id);
        
        let info = NowPlayingInfo {
            username: np.username,
            player_name: np.player_name,
            song_title: entry.as_ref().map(|(s, _, _)| s.title.clone()),
            artist_name: entry.as_ref().map(|(_, _, artists)| {
                artists.iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
            }).filter(|s| !s.is_empty()),
            album_name: entry.as_ref().and_then(|(_, album, _)| album.as_ref().map(|a| a.name.clone())),
            album_id: entry.as_ref().and_then(|(s, _, _)| s.album_id.clone()),
            cover_art: entry.as_ref().map(|(s, _, _)| {
                if let Some(aid) = &s.album_id {
                    format!("al-{}", aid)
                } else {
                    s.id.clone()
                }
            }),
            updated_at: np.updated_at,
        };
        
        now_playing_info.push(info);
    }

    Ok(Json(now_playing_info))
}

#[handler]
pub async fn get_system_info() -> Result<Json<SystemInfo>, poem::Error> {
    let (cpu_usage, memory_usage, memory_total) = {
        let mut sys = SYS.lock().map_err(|e| {
            log::error!("System mutex poisoned: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let pid = sysinfo::get_current_pid().map_err(|e| {
            log::error!("Failed to get current PID: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        
        let (cpu, mem) = if let Some(proc) = sys.process(pid) {
            (proc.cpu_usage(), proc.memory())
        } else {
            (0.0, 0)
        };
        
        (cpu, mem, sys.total_memory())
    };
    
    Ok(Json(SystemInfo {
        cpu_usage,
        memory_usage,
        memory_total,
    }))
}

#[handler]
pub async fn get_folders(
    db: Data<&DatabaseConnection>,
) -> Result<Json<Vec<FolderInfo>>, poem::Error> {
    let folders = music_folder::Entity::find()
        .all(*db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch music folders: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    // Optimization: Fetch all song counts in a single query to avoid N+1 problem
    let counts = child::Entity::find()
        .select_only()
        .column(child::Column::MusicFolderId)
        .column_as(child::Column::Id.count(), "song_count")
        .filter(child::Column::IsDir.eq(false))
        .group_by(child::Column::MusicFolderId)
        .into_tuple::<(i32, i64)>()
        .all(*db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch folder song counts: {}", e);
            poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    let count_map: HashMap<i32, i64> = counts.into_iter().collect();

    let folder_infos = folders
        .into_iter()
        .map(|folder| {
            let song_count = count_map.get(&folder.id).cloned().unwrap_or(0);
            FolderInfo {
                label: folder.name.clone().unwrap_or_else(|| folder.path.clone()),
                path: folder.path,
                song_count: song_count as u64,
            }
        })
        .collect();
    
    Ok(Json(folder_infos))
}
