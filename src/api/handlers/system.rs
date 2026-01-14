use poem::{handler, web::{Data, Json, Query}};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, ColumnTrait, QueryFilter, QuerySelect};
use crate::models::{child, album, artist, genre, music_folder};
use serde::{Serialize, Deserialize};
use sysinfo::System;
use std::{collections::{HashMap, HashSet}, sync::Mutex};
use once_cell::sync::Lazy;

static SYS: Lazy<Mutex<System>> = Lazy::new(|| {
    let mut sys = System::new_all();
    sys.refresh_all();
    Mutex::new(sys)
});

#[derive(Deserialize)]
pub struct StatsQuery {
    pub fields: Option<String>,
}

#[derive(Serialize, Default)]
pub struct Stats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub songs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albums: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artists: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<u64>,
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

#[handler]
pub async fn get_stats(
    db: Data<&DatabaseConnection>,
    query: Query<StatsQuery>,
) -> Result<Json<Stats>, poem::Error> {
    let field_set: HashSet<&str> = query.fields.as_deref()
        .map(|f| f.split(',').collect())
        .unwrap_or_default();
    let fetch_all = field_set.is_empty();

    let (songs, albums, artists, genres) = tokio::try_join!(
        async {
            if fetch_all || field_set.contains("songs") {
                child::Entity::find()
                    .filter(child::Column::IsDir.eq(false))
                    .count(*db)
                    .await
                    .map(Some)
            } else {
                Ok(None)
            }
        },
        async {
            if fetch_all || field_set.contains("albums") {
                album::Entity::find().count(*db).await.map(Some)
            } else {
                Ok(None)
            }
        },
        async {
            if fetch_all || field_set.contains("artists") {
                artist::Entity::find().count(*db).await.map(Some)
            } else {
                Ok(None)
            }
        },
        async {
            if fetch_all || field_set.contains("genres") {
                genre::Entity::find().count(*db).await.map(Some)
            } else {
                Ok(None)
            }
        },
    )
    .map_err(|e: sea_orm::DbErr| {
        log::error!("Failed to fetch stats: {}", e);
        poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    Ok(Json(Stats {
        songs,
        albums,
        artists,
        genres,
    }))
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
