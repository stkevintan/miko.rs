use poem::{handler, web::{Data, Json}, IntoResponse};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, ColumnTrait, QueryFilter, ModelTrait};
use crate::models::{child, album, artist, now_playing, playlist, user};
use serde::Serialize;
use sysinfo::System;
use std::sync::Mutex;
use once_cell::sync::Lazy;

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
pub struct UserInfo {
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
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
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct DashboardData {
    pub stats: Stats,
    pub system: SystemInfo,
    pub user: Option<UserInfo>,
    pub now_playing: Vec<NowPlayingInfo>,
}

#[handler]
pub async fn get_dashboard_data(
    db: Data<&DatabaseConnection>,
    user: Data<&user::Model>,
) -> impl IntoResponse {
    let songs = child::Entity::find()
        .filter(child::Column::IsDir.eq(false))
        .count(*db)
        .await
        .unwrap_or(0);
        
    let albums = album::Entity::find()
        .count(*db)
        .await
        .unwrap_or(0);
        
    let artists = artist::Entity::find()
        .count(*db)
        .await
        .unwrap_or(0);

    let playlists = playlist::Entity::find()
        .count(*db)
        .await
        .unwrap_or(0);

    let now_playing_list = now_playing::Entity::find()
        .all(*db)
        .await
        .unwrap_or_default();

    let mut now_playing_info = Vec::new();
    for np in now_playing_list {
        let song = child::Entity::find_by_id(&np.song_id).one(*db).await.ok().flatten();
        let mut info = NowPlayingInfo {
            username: np.username,
            player_name: np.player_name,
            song_title: song.as_ref().map(|s| s.title.clone()),
            artist_name: None,
            album_name: None,
            album_id: song.as_ref().and_then(|s| s.album_id.clone()),
            updated_at: np.updated_at,
        };

        if let Some(s) = song {
            // Get artist
            let artists = s.find_related(artist::Entity).all(*db).await.ok().unwrap_or_default();
            info.artist_name = artists.first().map(|a| a.name.clone());

            // Get album
            if let Some(album_id) = &s.album_id {
                let album = album::Entity::find_by_id(album_id).one(*db).await.ok().flatten();
                info.album_name = album.map(|a| a.name);
            }
        }
        now_playing_info.push(info);
    }

    // Extract user info from injected data
    let mut roles = Vec::new();
    if user.admin_role { roles.push("Admin".to_string()); }
    if user.settings_role { roles.push("Settings".to_string()); }
    if user.download_role { roles.push("Download".to_string()); }
    if user.upload_role { roles.push("Upload".to_string()); }
    if user.playlist_role { roles.push("Playlist".to_string()); }
    if user.cover_art_role { roles.push("Cover Art".to_string()); }
    if user.comment_role { roles.push("Comment".to_string()); }
    if user.podcast_role { roles.push("Podcast".to_string()); }
    if user.stream_role { roles.push("Stream".to_string()); }
    if user.jukebox_role { roles.push("Jukebox".to_string()); }
    if user.share_role { roles.push("Share".to_string()); }
    if user.video_conversion_role { roles.push("Video Conversion".to_string()); }

    let current_user_info = Some(UserInfo {
        username: user.username.clone(),
        email: user.email.clone(),
        roles,
    });

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

    Json(DashboardData {
        stats: Stats { songs, albums, artists, playlists },
        system: system_info,
        user: current_user_info,
        now_playing: now_playing_info,
    })
}
