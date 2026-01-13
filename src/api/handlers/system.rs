use poem::{handler, web::{Data, Json}};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, ColumnTrait, QueryFilter, LoaderTrait};
use crate::models::{child, album, artist, now_playing, playlist, song_artist};
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
    pub playlists: u64,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
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
) -> Json<Stats> {
    let (songs, albums, artists, playlists) = tokio::try_join!(
        child::Entity::find().filter(child::Column::IsDir.eq(false)).count(*db),
        album::Entity::find().count(*db),
        artist::Entity::find().count(*db),
        playlist::Entity::find().count(*db),
    ).unwrap_or((0, 0, 0, 0));

    Json(Stats { songs, albums, artists, playlists })
}

#[handler]
pub async fn get_now_playing(
    db: Data<&DatabaseConnection>,
) -> Json<Vec<NowPlayingInfo>> {
    let now_playing_list = now_playing::Entity::find()
        .all(*db)
        .await
        .unwrap_or_default();

    let song_ids: Vec<String> = now_playing_list.iter().map(|np| np.song_id.clone()).collect();
    
    let songs_with_albums = if !song_ids.is_empty() {
        child::Entity::find()
            .filter(child::Column::Id.is_in(song_ids))
            .find_also_related(album::Entity)
            .all(*db)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let song_models: Vec<child::Model> = songs_with_albums.iter().map(|(s, _)| s.clone()).collect();
    let artists_per_song = if !song_models.is_empty() {
        song_models.load_many_to_many(artist::Entity, song_artist::Entity, *db)
            .await
            .unwrap_or_default()
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
            artist_name: entry.as_ref().and_then(|(_, _, artists)| artists.first().map(|a| a.name.clone())),
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

    Json(now_playing_info)
}

#[handler]
pub fn get_system_info() -> Json<SystemInfo> {
    let system_info = {
        let mut sys = SYS.lock().unwrap();
        let pid = sysinfo::get_current_pid().unwrap();
        sys.refresh_all();
        
        let (cpu_usage, memory_usage) = if let Some(proc) = sys.process(pid) {
            (proc.cpu_usage(), proc.memory())
        } else {
            (0.0, 0)
        };
        
        SystemInfo {
            cpu_usage,
            memory_usage,
            memory_total: sys.total_memory(),
        }
    };

    Json(system_info)
}
