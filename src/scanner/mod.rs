use crate::config::Config;
use crate::models::{
    album, album_artist, album_genre, artist, child, genre, lyrics, music_folder, song_artist,
    song_genre,
};
use crate::scanner::walker::{WalkTask, Walker};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI64};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

pub mod tags;
pub mod utils;
pub mod walker;
pub mod seen;

pub struct SongRelations {
    pub song_id: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
    pub lyrics: Option<String>,
}

pub struct AlbumRelations {
    pub album_id: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
}

pub enum UpsertMessage {
    Artist(Box<artist::ActiveModel>),
    Album(Box<album::ActiveModel>),
    Genre(Box<genre::ActiveModel>),
    Song(Box<child::ActiveModel>),
    SongRelations(Box<SongRelations>),
    AlbumRelations(Box<AlbumRelations>),
    Seen(String),
    Flush(tokio::sync::oneshot::Sender<()>),
}

pub struct Scanner {
    db: DatabaseConnection,
    cfg: Arc<Config>,
    is_scanning: Arc<AtomicBool>,
    scan_count: Arc<AtomicI64>,
    total_count: Arc<AtomicI64>,
    last_scan_time: Arc<AtomicI64>,
    upsert_tx: mpsc::Sender<UpsertMessage>,
}

impl Clone for Scanner {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            cfg: self.cfg.clone(),
            is_scanning: self.is_scanning.clone(),
            scan_count: self.scan_count.clone(),
            total_count: self.total_count.clone(),
            last_scan_time: self.last_scan_time.clone(),
            upsert_tx: self.upsert_tx.clone(),
        }
    }
}

struct ScanGuard(Arc<AtomicBool>);

impl Drop for ScanGuard {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Scanner {
    pub fn new(db: DatabaseConnection, cfg: Arc<Config>) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        let flusher_db = db.clone();
        tokio::spawn(async move {
            Self::run_flusher(flusher_db, rx).await;
        });

        Self {
            db,
            cfg,
            is_scanning: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicI64::new(0)),
            total_count: Arc::new(AtomicI64::new(0)),
            last_scan_time: Arc::new(AtomicI64::new(0)),
            upsert_tx: tx,
        }
    }

    async fn run_flusher(db: DatabaseConnection, mut rx: mpsc::Receiver<UpsertMessage>) {
        let mut artists = Vec::new();
        let mut albums = Vec::new();
        let mut genres = Vec::new();
        let mut songs = Vec::new();
        let mut song_relations = Vec::new();
        let mut album_relations = Vec::new();
        let mut seen_ids = Vec::new();
        let mut flush_ack: Option<tokio::sync::oneshot::Sender<()>> = None;

        let flush_interval = std::time::Duration::from_millis(500);
        let mut last_flush = std::time::Instant::now();

        loop {
            let msg = tokio::select! {
                m = rx.recv() => m,
                _ = tokio::time::sleep(flush_interval) => None,
            };

            let mut force_flush = false;
            let is_none = msg.is_none();

            if let Some(m) = msg {
                match m {
                    UpsertMessage::Artist(v) => artists.push(*v),
                    UpsertMessage::Album(v) => albums.push(*v),
                    UpsertMessage::Genre(v) => genres.push(*v),
                    UpsertMessage::Song(v) => songs.push(*v),
                    UpsertMessage::SongRelations(v) => song_relations.push(*v),
                    UpsertMessage::AlbumRelations(v) => album_relations.push(*v),
                    UpsertMessage::Seen(v) => seen_ids.push(v),
                    UpsertMessage::Flush(tx) => {
                        force_flush = true;
                        flush_ack = Some(tx);
                    }
                }
            }

            let overdue = last_flush.elapsed() >= flush_interval || force_flush;
            
            if artists.len() >= 100 || (overdue && !artists.is_empty()) {
                let items = std::mem::take(&mut artists);
                if let Err(e) = artist::Entity::insert_many(items)
                    .on_conflict(sea_orm::sea_query::OnConflict::column(artist::Column::Id).do_nothing().to_owned())
                    .exec_without_returning(&db).await {
                    log::error!("Failed to flush artists: {}", e);
                }
            }
            if albums.len() >= 100 || (overdue && !albums.is_empty()) {
                let items = std::mem::take(&mut albums);
                if let Err(e) = album::Entity::insert_many(items)
                    .on_conflict(sea_orm::sea_query::OnConflict::column(album::Column::Id).update_columns([album::Column::Year]).to_owned())
                    .exec_without_returning(&db).await {
                    log::error!("Failed to flush albums: {}", e);
                }
            }
            if genres.len() >= 50 || (overdue && !genres.is_empty()) {
                let items = std::mem::take(&mut genres);
                if let Err(e) = genre::Entity::insert_many(items)
                    .on_conflict(sea_orm::sea_query::OnConflict::column(genre::Column::Name).do_nothing().to_owned())
                    .exec_without_returning(&db).await {
                    log::error!("Failed to flush genres: {}", e);
                }
            }
            if songs.len() >= 100 || (overdue && !songs.is_empty()) {
                let items = std::mem::take(&mut songs);
                if let Err(e) = child::Entity::insert_many(items)
                    .on_conflict(sea_orm::sea_query::OnConflict::column(child::Column::Id).update_columns([
                        child::Column::Parent, child::Column::Title, child::Column::Path, child::Column::Size, 
                        child::Column::Suffix, child::Column::ContentType, child::Column::Track, 
                        child::Column::DiscNumber, child::Column::Year, child::Column::Duration, 
                        child::Column::BitRate, child::Column::AlbumId
                    ]).to_owned())
                    .exec_without_returning(&db).await {
                    log::error!("Failed to flush songs: {}", e);
                }
            }
            if song_relations.len() >= 100 || (overdue && !song_relations.is_empty()) {
                let relations = std::mem::take(&mut song_relations);
                let song_ids: Vec<String> = relations.iter().map(|r| r.song_id.clone()).collect();
                
                let mut all_artists = Vec::new();
                let mut all_genres = Vec::new();
                let mut all_lyrics = Vec::new();
                
                for r in relations {
                    for a_id in r.artists {
                        all_artists.push(song_artist::ActiveModel {
                            song_id: Set(r.song_id.clone()),
                            artist_id: Set(a_id),
                        });
                    }
                    for g_name in r.genres {
                        all_genres.push(song_genre::ActiveModel {
                            song_id: Set(r.song_id.clone()),
                            genre_name: Set(g_name),
                        });
                    }
                    if let Some(content) = r.lyrics {
                        all_lyrics.push(lyrics::ActiveModel {
                            song_id: Set(r.song_id.clone()),
                            content: Set(content),
                        });
                    }
                }

                let flush_op = async {
                    let txn = db.begin().await?;
                    
                    song_artist::Entity::delete_many()
                        .filter(song_artist::Column::SongId.is_in(&song_ids))
                        .exec(&txn)
                        .await?;
                    song_genre::Entity::delete_many()
                        .filter(song_genre::Column::SongId.is_in(&song_ids))
                        .exec(&txn)
                        .await?;
                    lyrics::Entity::delete_many()
                        .filter(lyrics::Column::SongId.is_in(song_ids))
                        .exec(&txn)
                        .await?;

                    if !all_artists.is_empty() {
                        song_artist::Entity::insert_many(all_artists)
                            .on_conflict(sea_orm::sea_query::OnConflict::columns([song_artist::Column::SongId, song_artist::Column::ArtistId]).do_nothing().to_owned())
                            .exec_without_returning(&txn).await?;
                    }
                    if !all_genres.is_empty() {
                        song_genre::Entity::insert_many(all_genres)
                            .on_conflict(sea_orm::sea_query::OnConflict::columns([song_genre::Column::SongId, song_genre::Column::GenreName]).do_nothing().to_owned())
                            .exec_without_returning(&txn).await?;
                    }
                    if !all_lyrics.is_empty() {
                        lyrics::Entity::insert_many(all_lyrics)
                            .on_conflict(sea_orm::sea_query::OnConflict::column(lyrics::Column::SongId).do_nothing().to_owned())
                            .exec_without_returning(&txn).await?;
                    }
                    
                    txn.commit().await?;
                    Ok::<(), sea_orm::DbErr>(())
                };

                if let Err(e) = flush_op.await {
                    log::error!("Failed to flush song relations: {}", e);
                }
            }

            if album_relations.len() >= 100 || (overdue && !album_relations.is_empty()) {
                let relations = std::mem::take(&mut album_relations);
                let album_ids: Vec<String> = relations.iter().map(|r| r.album_id.clone()).collect();
                
                let mut all_artists = Vec::new();
                let mut all_genres = Vec::new();
                
                for r in relations {
                    for a_id in r.artists {
                        all_artists.push(album_artist::ActiveModel {
                            album_id: Set(r.album_id.clone()),
                            artist_id: Set(a_id),
                        });
                    }
                    for g_name in r.genres {
                        all_genres.push(album_genre::ActiveModel {
                            album_id: Set(r.album_id.clone()),
                            genre_name: Set(g_name),
                        });
                    }
                }

                let flush_op = async {
                    let txn = db.begin().await?;
                    
                    album_artist::Entity::delete_many()
                        .filter(album_artist::Column::AlbumId.is_in(&album_ids))
                        .exec(&txn)
                        .await?;
                    album_genre::Entity::delete_many()
                        .filter(album_genre::Column::AlbumId.is_in(album_ids))
                        .exec(&txn)
                        .await?;

                    if !all_artists.is_empty() {
                        album_artist::Entity::insert_many(all_artists)
                            .on_conflict(sea_orm::sea_query::OnConflict::columns([album_artist::Column::AlbumId, album_artist::Column::ArtistId]).do_nothing().to_owned())
                            .exec_without_returning(&txn).await?;
                    }
                    if !all_genres.is_empty() {
                        album_genre::Entity::insert_many(all_genres)
                            .on_conflict(sea_orm::sea_query::OnConflict::columns([album_genre::Column::AlbumId, album_genre::Column::GenreName]).do_nothing().to_owned())
                            .exec_without_returning(&txn).await?;
                    }
                    
                    txn.commit().await?;
                    Ok::<(), sea_orm::DbErr>(())
                };

                if let Err(e) = flush_op.await {
                    log::error!("Failed to flush album relations: {}", e);
                }
            }

            if seen_ids.len() >= 500 || (overdue && !seen_ids.is_empty()) {
                let ids = std::mem::take(&mut seen_ids);
                if let Err(e) = seen::SeenTracker::insert_batch(&db, ids).await {
                    log::error!("Failed to bulk insert seen IDs: {}", e);
                }
            }

            if overdue {
                last_flush = std::time::Instant::now();
                if let Some(tx) = flush_ack.take() {
                    let _ = tx.send(());
                }
            }

            if is_none && rx.is_closed() {
                break;
            }
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.is_scanning.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn last_scan_time(&self) -> i64 {
        self.last_scan_time
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn scan_count(&self) -> i64 {
        self.scan_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn total_count(&self) -> i64 {
        self.total_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn update_total_count(&self) {
        let count = child::Entity::count_songs(&self.db).await;
        self.total_count
            .store(count, std::sync::atomic::Ordering::SeqCst);
    }

    async fn process_task(
        &self,
        task: WalkTask,
        incremental: bool,
        cache_dir: &Path,
    ) -> Result<(), anyhow::Error> {
        let id = utils::generate_id(&task.path, task.folder.id, &task.folder.path);
        
        let parent_id = utils::get_parent_id(&task.path, task.folder.id, &task.folder.path)
            .filter(|s| !s.is_empty());

        self.upsert_tx.send(UpsertMessage::Seen(id.clone())).await?;

        if task.is_dir {
            let active_child: child::ActiveModel = child::ActiveModel {
                id: Set(id.clone()),
                parent: Set(parent_id),
                is_dir: Set(true),
                title: Set(task.name.clone()),
                path: Set(task.path.clone()),
                music_folder_id: Set(task.folder.id),
                content_type: Set(None),
                suffix: Set(None),
                transcoded_content_type: Set(None),
                transcoded_suffix: Set(None),
                album_id: Set(None),
                r#type: Set("directory".to_string()),
                track: Set(0),
                year: Set(0),
                disc_number: Set(0),
                duration: Set(0),
                bit_rate: Set(0),
                size: Set(0),
                is_video: Set(false),
                user_rating: Set(0),
                average_rating: Set(0.0),
                play_count: Set(0),
                ..Default::default()
            };
            self.upsert_tx.send(UpsertMessage::Song(Box::new(active_child))).await?;
            return Ok(());
        }

        if incremental {
            let existing: Option<chrono::DateTime<chrono::Utc>> = child::Entity::find_by_id(id.clone())
                .select_only()
                .column(child::Column::Created)
                .into_tuple()
                .one(&self.db)
                .await?;
            
            if let Some(created) = existing {
                if task.mod_time <= created {
                    return Ok(());
                }
            }
        }
        // file must be end with a valid audio suffix, that is ensured by the walker
        let content_type = format!("audio/{}", task.ext.to_lowercase());

        let path_for_tags = Path::new(&task.path).to_path_buf();
        let tag_data =
            match tokio::task::spawn_blocking(move || tags::read(&path_for_tags)).await? {
                Ok(t) => Some(t),
                Err(e) => {
                    log::warn!("Failed to read tags for '{}': {}", &task.path, e);
                    None
                }
            };

        let mut relations = SongRelations {
            song_id: id.clone(),
            artists: Vec::new(),
            genres: Vec::new(),
            lyrics: None,
        };

        let mut active_child = child::ActiveModel {
            id: Set(id.clone()),
            parent: Set(parent_id),
            is_dir: Set(false),
            title: Set(tag_data
                .as_ref()
                .filter(|t| !t.title.trim().is_empty())
                .map(|t| t.title.clone())
                .unwrap_or_else(|| task.name.clone())),
            path: Set(task.path.clone()),
            size: Set(task.size as i64),
            suffix: Set(Some(task.ext.clone())),
            content_type: Set(Some(content_type)),
            created: Set(Some(task.mod_time)),
            music_folder_id: Set(task.folder.id),
            transcoded_content_type: Set(None),
            transcoded_suffix: Set(None),
            album_id: Set(None),
            r#type: Set("music".to_string()),
            track: Set(0),
            year: Set(0),
            disc_number: Set(0),
            duration: Set(0),
            bit_rate: Set(0),
            is_video: Set(false),
            user_rating: Set(0),
            average_rating: Set(0.0),
            play_count: Set(0),
            ..Default::default()
        };

        if let Some(t) = tag_data {
            active_child.track = Set(t.track.unwrap_or(0));
            active_child.disc_number = Set(t.disc.unwrap_or(0));
            active_child.year = Set(t.year.unwrap_or(0));
            relations.lyrics = (!t.lyrics.trim().is_empty()).then(|| t.lyrics.clone());
            active_child.duration = Set(t.duration);
            active_child.bit_rate = Set(t.bitrate);

            let filtered_artists: Vec<&str> = t
                .artists
                .iter()
                .map(|a| a.trim())
                .filter(|a| !a.is_empty())
                .collect();

            for a_name in &filtered_artists {
                let a_id = self.ensure_artist(a_name).await?;
                relations.artists.push(a_id);
            }
            let mut album_artists_list: Vec<&str> = t
                .album_artists
                .iter()
                .map(|a| a.trim())
                .filter(|a| !a.is_empty())
                .collect();
            if album_artists_list.is_empty() {
                album_artists_list.push("Unknown Artist");
            }

            if !t.album.trim().is_empty() {
                let album_id = self
                    .ensure_album(
                        &t.album,
                        &album_artists_list,
                        t.year.unwrap_or(0),
                        &t.genres.iter().map(|g| g.as_str()).collect::<Vec<&str>>(),
                        task.mod_time,
                    )
                    .await?;
                active_child.album_id = Set(Some(album_id.clone()));
            }

            let filtered_genres: Vec<String> = t
                .genres
                .iter()
                .map(|g| g.trim())
                .filter(|g| !g.is_empty())
                .map(|g| g.to_string())
                .collect();

            for g_name in &filtered_genres {
                let g_name = self.ensure_genre(g_name).await?;
                relations.genres.push(g_name);
            }

            if t.has_image {
                let cover_art_id = if let Some(aid) = active_child.album_id.as_ref().as_ref() {
                    format!("al-{}", aid)
                } else {
                    id.clone()
                };
                let cover_path = cache_dir.join(&cover_art_id);
                if !cover_path.exists() {
                    let path_for_img = Path::new(&task.path).to_path_buf();
                    if let Ok(Ok(img_data)) =
                        tokio::task::spawn_blocking(move || tags::read_image(&path_for_img))
                            .await
                    {
                        tokio::fs::write(cover_path, img_data).await?;
                    }
                }
            }
        }

        self.upsert_tx.send(UpsertMessage::Song(Box::new(active_child))).await?;

        self.upsert_tx.send(UpsertMessage::SongRelations(Box::new(relations))).await?;

        self.scan_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        Ok(())
    }

    async fn ensure_artist(&self, name: &str) -> Result<String, anyhow::Error> {
        let id = utils::generate_artist_id(name);
        let obj = artist::ActiveModel {
            id: Set(id.clone()),
            name: Set(name.to_string()),
            artist_image_url: Set(None),
            user_rating: Set(0),
            average_rating: Set(0.0),
            ..Default::default()
        };
        self.upsert_tx.send(UpsertMessage::Artist(Box::new(obj))).await?;
        Ok(id)
    }

    async fn ensure_genre(&self, name: &str) -> Result<String, anyhow::Error> {
        let name = name.trim();
        let obj = genre::ActiveModel {
            name: Set(name.to_string()),
        };
        self.upsert_tx.send(UpsertMessage::Genre(Box::new(obj))).await?;
        Ok(name.to_string())
    }

    async fn ensure_album(
        &self,
        name: &str,
        artist_names: &[&str],
        year: i32,
        genres: &[&str],
        created: chrono::DateTime<chrono::Utc>,
    ) -> Result<String, anyhow::Error> {
        let artist_name = artist_names.join("; ");
        let id = utils::generate_album_id(&artist_name, name);

        let obj = album::ActiveModel {
            id: Set(id.clone()),
            name: Set(name.to_string()),
            created: Set(created),
            year: Set(year),
            user_rating: Set(0),
            average_rating: Set(0.0),
            ..Default::default()
        };
        self.upsert_tx.send(UpsertMessage::Album(Box::new(obj))).await?;

        let mut relations = AlbumRelations {
            album_id: id.clone(),
            artists: Vec::new(),
            genres: Vec::new(),
        };

        for &a_name in artist_names {
            relations.artists.push(self.ensure_artist(a_name).await?);
        }

        for &g_name in genres {
            let g_name = g_name.trim();
            if !g_name.is_empty() {
                relations.genres.push(self.ensure_genre(g_name).await?);
            }
        }

        self.upsert_tx.send(UpsertMessage::AlbumRelations(Box::new(relations))).await?;

        Ok(id)
    }

    pub async fn scan_all(&self, incremental: bool) -> Result<(), anyhow::Error> {
        if self
            .is_scanning
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            )
            .is_err()
        {
            return Ok(());
        }

        let _guard = ScanGuard(self.is_scanning.clone());
        self.scan_count
            .store(0, std::sync::atomic::Ordering::SeqCst);

        let (tx, mut rx) = mpsc::channel(100);
        let folders = music_folder::Entity::find().all(&self.db).await?;

        for folder in folders {
            Walker::walk_path(Path::new(&folder.path).to_path_buf(), folder, tx.clone());
        }
        drop(tx);

        log::info!("Starting scan... incremental: {}", incremental);
        self.scan_count
            .store(0, std::sync::atomic::Ordering::SeqCst);

        // create a temporary table to track seen ids
        seen::SeenTracker::prepare(&self.db).await?;

        let cache_dir = utils::get_cover_cache_dir(&self.cfg);
        if !cache_dir.exists() {
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        let mut join_set = tokio::task::JoinSet::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(32));

        while let Some(task) = rx.recv().await {
            let scanner = self.clone();
            let cache_dir_clone = cache_dir.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            
            join_set.spawn(async move {
                let _permit = permit;
                if let Err(e) = scanner.process_task(task, incremental, &cache_dir_clone).await {
                    log::error!("Error processing task: {}", e);
                }
            });

            // Clean up finished tasks periodically
            while join_set.try_join_next().is_some() {}
        }

        while let Some(res) = join_set.join_next().await {
            if let Err(e) = res {
                log::error!("Join error: {}", e);
            }
        }

        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        self.upsert_tx.send(UpsertMessage::Flush(ack_tx)).await?;
        // Wait for the flusher to finish its work before pruning
        let _ = ack_rx.await;

        log::info!("Scan finished, pruning database...");
        self.prune().await?;

        self.update_total_count().await;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        self.last_scan_time
            .store(now, std::sync::atomic::Ordering::SeqCst);
        log::info!(
            "Scan completed. Total files: {}",
            self.scan_count.load(std::sync::atomic::Ordering::SeqCst)
        );

        Ok(())
    }

    pub async fn prune(&self) -> Result<(), anyhow::Error> {
        log::info!("Pruning deleted files and orphaned records...");

        // 1. Delete associated data for songs that are NOT in _scanner_seen
        // Order matters: delete dependents first to satisfy foreign key constraints.
        self.db.execute_unprepared("DELETE FROM lyrics WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = lyrics.song_id)").await?;
        self.db.execute_unprepared("DELETE FROM song_artists WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_artists.song_id)").await?;
        self.db.execute_unprepared("DELETE FROM song_genres WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_genres.song_id)").await?;
        self.db.execute_unprepared("DELETE FROM playlist_songs WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = playlist_songs.song_id)").await?;

        // 2. Delete children that are NOT in _scanner_seen
        self.db.execute_unprepared("DELETE FROM children WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = children.id)").await?;

        // 3. Prune orphaned albums (no more songs referencing them)
        // First delete junction records for those albums
        self.db.execute_unprepared("DELETE FROM album_artists WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = album_artists.album_id)").await?;
        self.db.execute_unprepared("DELETE FROM album_genres WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = album_genres.album_id)").await?;
        self.db.execute_unprepared("DELETE FROM albums WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = albums.id)").await?;

        // 4. Prune orphaned artists
        self.db.execute_unprepared("DELETE FROM artists \
            WHERE NOT EXISTS (SELECT 1 FROM song_artists WHERE song_artists.artist_id = artists.id) \
            AND NOT EXISTS (SELECT 1 FROM album_artists WHERE album_artists.artist_id = artists.id)").await?;

        // 5. Prune orphaned genres
        self.db.execute_unprepared("DELETE FROM genres \
            WHERE NOT EXISTS (SELECT 1 FROM album_genres WHERE album_genres.genre_name = genres.name) \
            AND NOT EXISTS (SELECT 1 FROM song_genres WHERE song_genres.genre_name = genres.name)").await?;

        // Cleanup side table
        seen::SeenTracker::clear(&self.db).await?;

        Ok(())
    }
}
