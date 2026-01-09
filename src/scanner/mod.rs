use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, QuerySelect, ConnectionTrait};
use crate::config::Config;
use crate::models::{child, music_folder, artist, album, album_artist, genre, song_artist, song_genre, lyrics};
use crate::scanner::walker::{Walker, WalkTask};
use std::path::Path;
use std::collections::{HashMap, HashSet};

pub mod walker;
pub mod tags;
pub mod utils;

pub struct Scanner {
    db: DatabaseConnection,
    cfg: Arc<Config>,
    is_scanning: AtomicBool,
    scan_count: AtomicI64,
    total_count: AtomicI64,
    last_scan_time: AtomicI64,
}

struct ScanGuard<'a>(&'a AtomicBool);

impl<'a> Drop for ScanGuard<'a> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Scanner {
    pub fn new(db: DatabaseConnection, cfg: Arc<Config>) -> Self {
        Self {
            db,
            cfg,
            is_scanning: AtomicBool::new(false),
            scan_count: AtomicI64::new(0),
            total_count: AtomicI64::new(0),
            last_scan_time: AtomicI64::new(0),
        }
    }

    pub fn is_scanning(&self) -> bool {
        self.is_scanning.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn last_scan_time(&self) -> i64 {
        self.last_scan_time.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn scan_count(&self) -> i64 {
        self.scan_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn total_count(&self) -> i64 {
        self.total_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn update_total_count(&self) {
        let count = child::Entity::count_songs(&self.db).await;
        self.total_count.store(count, std::sync::atomic::Ordering::SeqCst);
    }

    async fn process_batch(
        &self,
        tasks: &[WalkTask],
        incremental: bool,
        cache_dir: &Path,
        seen_artists: &mut HashSet<String>,
        seen_albums: &mut HashSet<String>,
        seen_genres: &mut HashSet<String>,
    ) -> Result<(), anyhow::Error> {
        let mut ids = Vec::new();
        let mut task_map = HashMap::new();

        for task in tasks {
            let id = utils::generate_id(&task.path, task.folder.id, &task.folder.path);
            ids.push(id.clone());
            task_map.insert(id, task);
        }

        let mut existing_info = HashMap::new();
        if incremental {
            let existing: Vec<(String, Option<chrono::DateTime<chrono::Utc>>)> = child::Entity::find()
                .filter(child::Column::Id.is_in(ids.clone()))
                .select_only()
                .column(child::Column::Id)
                .column(child::Column::Created)
                .into_tuple()
                .all(&self.db)
                .await?;
            for (id, created) in existing {
                if let Some(c) = created {
                    existing_info.insert(id, c);
                }
            }
        }

        let mut seen_ids_to_flush = Vec::new();

        for id in &ids {
            let task = task_map.get(id).unwrap();
            let parent_id = utils::get_parent_id(&task.path, task.folder.id, &task.folder.path)
                .filter(|s| !s.is_empty());
            
            seen_ids_to_flush.push(id.clone());

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
                child::Entity::insert(active_child).on_conflict(
                    sea_orm::sea_query::OnConflict::column(child::Column::Id)
                        .update_columns([child::Column::Title, child::Column::Parent, child::Column::Path])
                        .to_owned()
                ).exec_without_returning(&self.db).await?;
                continue;
            }

            if incremental {
                if let Some(last_mod) = existing_info.get(id) {
                    if task.mod_time <= *last_mod {
                        continue;
                    }
                }
            }

            let content_type = utils::get_content_type(Path::new(&task.path));
            let suffix = Path::new(&task.path).extension().and_then(|s| s.to_str()).unwrap_or_default().to_string();

            let path_for_tags = Path::new(&task.path).to_path_buf();
            let tag_data = match tokio::task::spawn_blocking(move || tags::read(&path_for_tags)).await? {
                Ok(t) => Some(t),
                Err(e) => {
                    log::warn!("Failed to read tags for '{}': {}", &task.path, e);
                    None
                }
            };

            let mut many_to_many_artists = Vec::new();
            let mut many_to_many_genres = Vec::new();
            let mut song_lyrics = None;

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
                suffix: Set(Some(suffix)),
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
                song_lyrics = (!t.lyrics.trim().is_empty()).then(|| t.lyrics.clone());
                active_child.duration = Set(t.duration);
                active_child.bit_rate = Set(t.bitrate);

                let mut new_artists = Vec::new();
                let filtered_artists: Vec<String> = t.artists.iter()
                    .map(|a| a.trim())
                    .filter(|a| !a.is_empty())
                    .map(|a| a.to_string())
                    .collect();

                for a_name in &filtered_artists {
                    let a_id = utils::generate_artist_id(a_name);
                    many_to_many_artists.push(a_id.clone());
                    if !seen_artists.contains(&a_id) {
                        new_artists.push(artist::ActiveModel {
                            id: Set(a_id.clone()),
                            name: Set(a_name.clone()),
                            artist_image_url: Set(None),
                            user_rating: Set(0),
                            average_rating: Set(0.0),
                            ..Default::default()
                        });
                        seen_artists.insert(a_id);
                    }
                }

                if !new_artists.is_empty() {
                    artist::Entity::insert_many(new_artists)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::column(artist::Column::Id)
                                .do_nothing()
                                .to_owned(),
                        )
                        .exec_without_returning(&self.db)
                        .await?;
                }

                if !t.album.trim().is_empty() {
                    let album_artists = if filtered_artists.is_empty() {
                        let filtered_album_artists: Vec<String> = t.album_artists.iter()
                            .map(|a| a.trim())
                            .filter(|a| !a.is_empty())
                            .map(|a| a.to_string())
                            .collect();
                        if filtered_album_artists.is_empty() {
                            vec!["Unknown Artist".to_string()]
                        } else {
                            filtered_album_artists
                        }
                    } else {
                        filtered_artists.clone()
                    };
                    let album_id = self
                        .ensure_album(
                            t.album.clone(),
                            album_artists,
                            t.year.unwrap_or(0),
                            t.genre.clone(),
                            task.mod_time,
                            seen_artists,
                            seen_albums,
                        )
                        .await?;
                    active_child.album_id = Set(Some(album_id.clone()));
                }

                let mut new_genres = Vec::new();
                let filtered_genres: Vec<String> = t.genres.iter()
                    .map(|g| g.trim())
                    .filter(|g| !g.is_empty())
                    .map(|g| g.to_string())
                    .collect();

                for g_name in &filtered_genres {
                    many_to_many_genres.push(g_name.clone());
                    if !seen_genres.contains(g_name) {
                        new_genres.push(genre::ActiveModel {
                            name: Set(g_name.clone()),
                        });
                        seen_genres.insert(g_name.clone());
                    }
                }

                if !new_genres.is_empty() {
                    genre::Entity::insert_many(new_genres)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::column(genre::Column::Name)
                                .do_nothing()
                                .to_owned(),
                        )
                        .exec_without_returning(&self.db)
                        .await?;
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
                        if let Ok(Ok(img_data)) = tokio::task::spawn_blocking(move || tags::read_image(&path_for_img)).await {
                            tokio::fs::write(cover_path, img_data).await?;
                        }
                    }
                }
            }

            child::Entity::insert(active_child).on_conflict(
                sea_orm::sea_query::OnConflict::column(child::Column::Id)
                    .update_columns([
                        child::Column::Parent,
                        child::Column::Title,
                        child::Column::Path,
                        child::Column::Size,
                        child::Column::Suffix,
                        child::Column::ContentType,
                        child::Column::Track,
                        child::Column::DiscNumber,
                        child::Column::Year,
                        child::Column::Duration,
                        child::Column::BitRate,
                        child::Column::AlbumId,
                    ])
                    .to_owned()
            ).exec_without_returning(&self.db).await?;

            // Clear existing many-to-many before re-inserting
            song_artist::Entity::delete_many()
                .filter(song_artist::Column::SongId.eq(id.clone()))
                .exec(&self.db)
                .await?;
            song_genre::Entity::delete_many()
                .filter(song_genre::Column::SongId.eq(id.clone()))
                .exec(&self.db)
                .await?;

            // Handle lyrics
            lyrics::Entity::delete_many()
                .filter(lyrics::Column::SongId.eq(id.clone()))
                .exec(&self.db)
                .await?;
            if let Some(content) = song_lyrics {
                lyrics::Entity::insert(lyrics::ActiveModel {
                    song_id: Set(id.clone()),
                    content: Set(content),
                }).exec_without_returning(&self.db).await?;
            }

            if !many_to_many_artists.is_empty() {
                song_artist::Entity::insert_many(many_to_many_artists.into_iter().map(|a_id| {
                    song_artist::ActiveModel {
                        song_id: Set(id.clone()),
                        artist_id: Set(a_id),
                    }
                }))
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        song_artist::Column::SongId,
                        song_artist::Column::ArtistId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(&self.db)
                .await?;
            }

            if !many_to_many_genres.is_empty() {
                song_genre::Entity::insert_many(many_to_many_genres.into_iter().map(|g_name| {
                    song_genre::ActiveModel {
                        song_id: Set(id.clone()),
                        genre_name: Set(g_name),
                    }
                }))
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        song_genre::Column::SongId,
                        song_genre::Column::GenreName,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(&self.db)
                .await?;
            }

            self.scan_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }

        self.flush_seen_ids(&seen_ids_to_flush).await?;

        Ok(())
    }

    async fn flush_seen_ids(&self, ids: &[String]) -> Result<(), anyhow::Error> {
        if ids.is_empty() {
            return Ok(());
        }
        use sea_orm::{Statement, TransactionTrait};
        let txn = self.db.begin().await?;
        for id in ids {
            txn.execute(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                "INSERT OR IGNORE INTO _scanner_seen (id) VALUES (?)",
                [sea_orm::Value::from(id.clone())],
            ))
            .await?;
        }
        txn.commit().await?;
        Ok(())
    }

    async fn ensure_artist(&self, name: &str, seen: &mut HashSet<String>) -> Result<String, anyhow::Error> {
        let id = utils::generate_artist_id(name);
        if !seen.contains(&id) {
            let obj = artist::ActiveModel {
                id: Set(id.clone()),
                name: Set(name.to_string()),
                artist_image_url: Set(None),
                user_rating: Set(0),
                average_rating: Set(0.0),
                ..Default::default()
            };
            artist::Entity::insert(obj).on_conflict(
                sea_orm::sea_query::OnConflict::column(artist::Column::Id)
                    .do_nothing()
                    .to_owned()
            ).exec_without_returning(&self.db).await?;
            seen.insert(id.clone());
        }
        Ok(id)
    }

    async fn ensure_album(
        &self,
        name: String,
        artist_names: Vec<String>,
        year: i32,
        genre: String,
        created: chrono::DateTime<chrono::Utc>,
        seen_artists: &mut HashSet<String>,
        seen_albums: &mut HashSet<String>,
    ) -> Result<String, anyhow::Error> {
        let artist_name = artist_names.join("; ");
        let id = utils::generate_album_id(&artist_name, &name);

        if !seen_albums.contains(&id) {
            // get first artist as main artist
            let main_artist = if artist_names.is_empty() {
                "Unknown Artist".to_string()
            } else {
                artist_names[0].clone()
            };

            let artist_id = self.ensure_artist(&main_artist, seen_artists).await?;

            let obj = album::ActiveModel {
                id: Set(id.clone()),
                name: Set(name),
                artist: Set((!artist_name.trim().is_empty()).then(|| artist_name.clone())),
                artist_id: Set(Some(artist_id)),
                created: Set(created),
                year: Set(year),
                genre: Set((!genre.trim().is_empty()).then(|| genre)),
                user_rating: Set(0),
                average_rating: Set(0.0),
                ..Default::default()
            };
            album::Entity::insert(obj)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(album::Column::Id)
                        .update_columns([album::Column::Year, album::Column::Genre])
                        .to_owned(),
                )
                .exec_without_returning(&self.db)
                .await?;

            let mut album_artist_models = Vec::new();
            for a_name in &artist_names {
                if let Ok(a_id) = self.ensure_artist(a_name, seen_artists).await {
                    album_artist_models.push(album_artist::ActiveModel {
                        album_id: Set(id.clone()),
                        artist_id: Set(a_id),
                    });
                }
            }

            if !album_artist_models.is_empty() {
                album_artist::Entity::insert_many(album_artist_models)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::columns([
                            album_artist::Column::AlbumId,
                            album_artist::Column::ArtistId,
                        ])
                        .do_nothing()
                        .to_owned(),
                    )
                    .exec_without_returning(&self.db)
                    .await?;
            }

            seen_albums.insert(id.clone());
        }
        Ok(id)
    }

    pub async fn scan_all(&self, incremental: bool) -> Result<(), anyhow::Error> {
        if self.is_scanning.compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_err() {
            return Ok(());
        }

        let _guard = ScanGuard(&self.is_scanning);
        self.scan_count.store(0, std::sync::atomic::Ordering::SeqCst);

        let (tx, mut rx) = mpsc::channel(100);
        let folders = music_folder::Entity::find().all(&self.db).await?;
        
        for folder in folders {
            Walker::walk_path(Path::new(&folder.path).to_path_buf(), folder, tx.clone());
        }
        drop(tx);

        log::info!("Starting scan... incremental: {}", incremental);
        self.scan_count.store(0, std::sync::atomic::Ordering::SeqCst);

        self.db.execute_unprepared("CREATE TABLE IF NOT EXISTS _scanner_seen (id TEXT PRIMARY KEY)").await?;
        self.db.execute_unprepared("DELETE FROM _scanner_seen").await?;

        let cache_dir = utils::get_cover_cache_dir(&self.cfg);
        if !cache_dir.exists() {
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        let mut seen_artists = HashSet::new();
        let mut seen_albums = HashSet::new();
        let mut seen_genres = HashSet::new();

        let mut task_buffer = Vec::with_capacity(100);
        while let Some(task) = rx.recv().await {
            task_buffer.push(task);
            if task_buffer.len() >= 100 {
                self.process_batch(
                    &task_buffer,
                    incremental,
                    &cache_dir,
                    &mut seen_artists,
                    &mut seen_albums,
                    &mut seen_genres,
                ).await?;
                task_buffer.clear();
            }
        }

        if !task_buffer.is_empty() {
            self.process_batch(
                &task_buffer,
                incremental,
                &cache_dir,
                &mut seen_artists,
                &mut seen_albums,
                &mut seen_genres,
            ).await?;
        }

        log::info!("Scan finished, pruning database...");
        self.prune().await?;

        self.update_total_count().await;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        self.last_scan_time.store(now, std::sync::atomic::Ordering::SeqCst);
        log::info!("Scan completed. Total files: {}", self.scan_count.load(std::sync::atomic::Ordering::SeqCst));

        Ok(())
    }

    pub async fn prune(&self) -> Result<(), anyhow::Error> {
        log::info!("Pruning deleted files and orphaned records...");

        // Delete junction records for songs that are NOT in _scanner_seen
        self.db.execute_unprepared("DELETE FROM song_artists WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_artists.song_id)").await?;
        self.db.execute_unprepared("DELETE FROM song_genres WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_genres.song_id)").await?;

        // Delete children that are NOT in _scanner_seen
        self.db.execute_unprepared("DELETE FROM children WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = children.id)").await?;

        // Drop the side table but keep it for next scan
        self.db.execute_unprepared("DELETE FROM _scanner_seen").await?;

        // 2. Prune orphaned albums
        self.db.execute_unprepared("DELETE FROM albums WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = albums.id)").await?;
        
        // 3. Prune orphaned artists
        self.db.execute_unprepared("DELETE FROM artists \
            WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.artist_id = artists.id) \
            AND NOT EXISTS (SELECT 1 FROM albums WHERE albums.artist_id = artists.id) \
            AND NOT EXISTS (SELECT 1 FROM song_artists WHERE song_artists.artist_id = artists.id)").await?;
        
        // 4. Prune orphaned genres
        self.db.execute_unprepared("DELETE FROM genres \
            WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.genre = genres.name) \
            AND NOT EXISTS (SELECT 1 FROM song_genres WHERE song_genres.genre_name = genres.name)").await?;

        Ok(())
    }
}
