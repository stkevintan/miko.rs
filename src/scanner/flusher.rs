use crate::models::{
    album, album_artist, album_genre, artist, child, genre, lyrics, song_artist, song_genre,
};
use crate::scanner::seen;
use crate::scanner::types::{AlbumRelations, SongRelations, UpsertMessage};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::time::{Duration, Instant};

/// Maximum rows per INSERT statement.
/// SQLite has a 32766 bind-parameter limit. The widest table (children) has ~20 columns,
/// so 500 rows × 20 cols = 10000 params — well within the limit.
const CHUNK_SIZE: usize = 500;

/// Extension trait to split a `Vec<T>` into owned chunks without requiring `Clone`.
trait IntoChunks {
    type Item;
    fn chunks_into(self, size: usize) -> Vec<Vec<Self::Item>>;
}

impl<T> IntoChunks for Vec<T> {
    type Item = T;
    fn chunks_into(mut self, size: usize) -> Vec<Vec<T>> {
        let mut result = Vec::with_capacity((self.len() + size - 1) / size);
        while !self.is_empty() {
            let end = self.len().min(size);
            result.push(self.drain(..end).collect());
        }
        result
    }
}

/// Flush artists (no dependencies).
async fn flush_artists<C: ConnectionTrait>(
    db: &C,
    items: Vec<artist::ActiveModel>,
) -> Result<(), sea_orm::DbErr> {
    for chunk in items.chunks_into(CHUNK_SIZE) {
        artist::Entity::insert_many(chunk)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(artist::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }
    Ok(())
}

/// Flush genres (no dependencies).
async fn flush_genres<C: ConnectionTrait>(
    db: &C,
    items: Vec<genre::ActiveModel>,
) -> Result<(), sea_orm::DbErr> {
    for chunk in items.chunks_into(CHUNK_SIZE) {
        genre::Entity::insert_many(chunk)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(genre::Column::Name)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }
    Ok(())
}

/// Flush albums (no dependencies on artists/genres in the table itself).
async fn flush_albums<C: ConnectionTrait>(
    db: &C,
    items: Vec<album::ActiveModel>,
) -> Result<(), sea_orm::DbErr> {
    for chunk in items.chunks_into(CHUNK_SIZE) {
        album::Entity::insert_many(chunk)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(album::Column::Id)
                    .update_columns([album::Column::Year])
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }
    Ok(())
}

/// Sort songs so directories come first, ordered by path length (shorter first).
/// This guarantees parent directories are inserted before their children,
/// satisfying the self-referencing parent FK within each batch.
fn sort_songs_for_insert(items: &mut [child::ActiveModel]) {
    items.sort_by(|a, b| {
        let a_dir = *a.is_dir.as_ref();
        let b_dir = *b.is_dir.as_ref();
        b_dir
            .cmp(&a_dir)
            .then_with(|| a.path.as_ref().len().cmp(&b.path.as_ref().len()))
    });
}

/// Flush songs/children (depends on albums via album_id FK, self-referencing parent FK).
/// Items are sorted so directories come first, ordered by path depth, ensuring parent
/// directories exist before their children within each batch.
async fn flush_songs<C: ConnectionTrait>(
    db: &C,
    mut items: Vec<child::ActiveModel>,
) -> Result<(), sea_orm::DbErr> {
    sort_songs_for_insert(&mut items);
    for chunk in items.chunks_into(CHUNK_SIZE) {
        child::Entity::insert_many(chunk)
            .on_conflict(
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
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }
    Ok(())
}

/// Flush song relations (depends on children, artists, genres).
async fn flush_song_relations<C: ConnectionTrait>(
    db: &C,
    relations: Vec<SongRelations>,
) -> Result<(), sea_orm::DbErr> {
    if relations.is_empty() {
        return Ok(());
    }
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

    song_artist::Entity::delete_many()
        .filter(song_artist::Column::SongId.is_in(&song_ids))
        .exec(db)
        .await?;
    song_genre::Entity::delete_many()
        .filter(song_genre::Column::SongId.is_in(&song_ids))
        .exec(db)
        .await?;
    lyrics::Entity::delete_many()
        .filter(lyrics::Column::SongId.is_in(&song_ids))
        .exec(db)
        .await?;

    if !all_artists.is_empty() {
        for chunk in all_artists.chunks_into(CHUNK_SIZE) {
            song_artist::Entity::insert_many(chunk)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        song_artist::Column::SongId,
                        song_artist::Column::ArtistId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(db)
                .await?;
        }
    }
    if !all_genres.is_empty() {
        for chunk in all_genres.chunks_into(CHUNK_SIZE) {
            song_genre::Entity::insert_many(chunk)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        song_genre::Column::SongId,
                        song_genre::Column::GenreName,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(db)
                .await?;
        }
    }
    if !all_lyrics.is_empty() {
        for chunk in all_lyrics.chunks_into(CHUNK_SIZE) {
            lyrics::Entity::insert_many(chunk)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(lyrics::Column::SongId)
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(db)
                .await?;
        }
    }

    Ok(())
}

/// Flush album relations (depends on albums, artists, genres).
async fn flush_album_relations<C: ConnectionTrait>(
    db: &C,
    relations: Vec<AlbumRelations>,
) -> Result<(), sea_orm::DbErr> {
    if relations.is_empty() {
        return Ok(());
    }
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

    album_artist::Entity::delete_many()
        .filter(album_artist::Column::AlbumId.is_in(&album_ids))
        .exec(db)
        .await?;
    album_genre::Entity::delete_many()
        .filter(album_genre::Column::AlbumId.is_in(&album_ids))
        .exec(db)
        .await?;
    if !all_artists.is_empty() {
        for chunk in all_artists.chunks_into(CHUNK_SIZE) {
            album_artist::Entity::insert_many(chunk)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        album_artist::Column::AlbumId,
                        album_artist::Column::ArtistId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(db)
                .await?;
        }
    }
    if !all_genres.is_empty() {
        for chunk in all_genres.chunks_into(CHUNK_SIZE) {
            album_genre::Entity::insert_many(chunk)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        album_genre::Column::AlbumId,
                        album_genre::Column::GenreName,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(db)
                .await?;
        }
    }

    Ok(())
}

/// Flush seen IDs.
async fn flush_seen<C: ConnectionTrait>(db: &C, ids: Vec<String>) -> Result<(), sea_orm::DbErr> {
    if ids.is_empty() {
        return Ok(());
    }
    let models: Vec<seen::ActiveModel> = ids
        .into_iter()
        .map(|id| seen::ActiveModel { id: Set(id) })
        .collect();
    for chunk in models.chunks_into(CHUNK_SIZE) {
        seen::Entity::insert_many(chunk)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(seen::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;
    }
    Ok(())
}

/// Execute a complete flush cycle within a single transaction.
/// Uses deferred foreign key constraints so all cross-table references
/// are resolved at commit time, preventing partial-flush FK failures.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn do_flush_cycle(
    db: &DatabaseConnection,
    artists: &mut Vec<artist::ActiveModel>,
    genres: &mut Vec<genre::ActiveModel>,
    albums: &mut Vec<album::ActiveModel>,
    songs: &mut Vec<child::ActiveModel>,
    song_relations: &mut Vec<SongRelations>,
    album_relations: &mut Vec<AlbumRelations>,
    seen_ids: &mut Vec<String>,
) -> Result<(), sea_orm::DbErr> {
    let a = std::mem::take(artists);
    let g = std::mem::take(genres);
    let al = std::mem::take(albums);
    let s = std::mem::take(songs);
    let sr = std::mem::take(song_relations);
    let ar = std::mem::take(album_relations);
    let si = std::mem::take(seen_ids);

    let txn = db.begin().await?;

    if db.get_database_backend() == sea_orm::DbBackend::Sqlite {
        // Defer FK constraint checks until COMMIT so that cross-table inserts
        // within this flush cycle don't fail due to insertion ordering.
        txn.execute_unprepared("PRAGMA defer_foreign_keys = ON")
            .await?;
    }

    // Flush in strict dependency order:
    //   1. artists, genres   (no FK dependencies)
    //   2. albums            (no FK deps on artists/genres directly)
    //   3. songs/children    (FK → albums.id, self-referencing parent)
    //   4. song_relations    (FK → children.id, artists.id, genres.name)
    //   5. album_relations   (FK → albums.id, artists.id, genres.name)
    //   6. seen ids          (independent)
    flush_artists(&txn, a).await?;
    flush_genres(&txn, g).await?;
    flush_albums(&txn, al).await?;
    flush_songs(&txn, s).await?;
    flush_song_relations(&txn, sr).await?;
    flush_album_relations(&txn, ar).await?;
    flush_seen(&txn, si).await?;

    txn.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn dispatch(
    msg: UpsertMessage,
    artists: &mut Vec<artist::ActiveModel>,
    albums: &mut Vec<album::ActiveModel>,
    genres: &mut Vec<genre::ActiveModel>,
    songs: &mut Vec<child::ActiveModel>,
    song_relations: &mut Vec<SongRelations>,
    album_relations: &mut Vec<AlbumRelations>,
    seen_ids: &mut Vec<String>,
    force_flush: &mut bool,
    flush_ack: &mut Option<tokio::sync::oneshot::Sender<()>>,
) {
    match msg {
        UpsertMessage::Artist(v) => artists.push(*v),
        UpsertMessage::Album(v) => albums.push(*v),
        UpsertMessage::Genre(v) => genres.push(*v),
        UpsertMessage::Song(v) => songs.push(*v),
        UpsertMessage::SongRelations(v) => song_relations.push(*v),
        UpsertMessage::AlbumRelations(v) => album_relations.push(*v),
        UpsertMessage::Seen(v) => seen_ids.push(v),
        UpsertMessage::Flush(tx) => {
            *force_flush = true;
            *flush_ack = Some(tx);
        }
        UpsertMessage::Batch(items) => {
            for item in items {
                dispatch(
                    item,
                    artists,
                    albums,
                    genres,
                    songs,
                    song_relations,
                    album_relations,
                    seen_ids,
                    force_flush,
                    flush_ack,
                );
            }
        }
    }
}

pub async fn run_flusher(
    db: DatabaseConnection,
    mut rx: tokio::sync::mpsc::Receiver<UpsertMessage>,
) {
    let mut artists = Vec::new();
    let mut albums = Vec::new();
    let mut genres = Vec::new();
    let mut songs = Vec::new();
    let mut song_relations = Vec::new();
    let mut album_relations = Vec::new();
    let mut seen_ids = Vec::new();
    let mut flush_ack: Option<tokio::sync::oneshot::Sender<()>> = None;

    let flush_interval = Duration::from_millis(500);
    let mut last_flush = Instant::now();

    loop {
        let msg = tokio::select! {
            m = rx.recv() => m,
            _ = tokio::time::sleep(flush_interval) => None,
        };

        let mut force_flush = false;
        let is_none = msg.is_none();

        if let Some(m) = msg {
            dispatch(
                m,
                &mut artists,
                &mut albums,
                &mut genres,
                &mut songs,
                &mut song_relations,
                &mut album_relations,
                &mut seen_ids,
                &mut force_flush,
                &mut flush_ack,
            );
        }

        // Drain all currently buffered messages without blocking.
        // This accumulates many messages per flush cycle, greatly reducing DB round-trips.
        while let Ok(m) = rx.try_recv() {
            dispatch(
                m,
                &mut artists,
                &mut albums,
                &mut genres,
                &mut songs,
                &mut song_relations,
                &mut album_relations,
                &mut seen_ids,
                &mut force_flush,
                &mut flush_ack,
            );
        }

        let overdue = last_flush.elapsed() >= flush_interval || force_flush;

        let any_threshold = artists.len() >= 100
            || genres.len() >= 50
            || albums.len() >= 100
            || songs.len() >= 100
            || song_relations.len() >= 100
            || album_relations.len() >= 100
            || seen_ids.len() >= 500;

        let has_data = !artists.is_empty()
            || !genres.is_empty()
            || !albums.is_empty()
            || !songs.is_empty()
            || !song_relations.is_empty()
            || !album_relations.is_empty()
            || !seen_ids.is_empty();

        let should_flush = any_threshold || (overdue && has_data) || force_flush;

        if should_flush {
            if let Err(e) = do_flush_cycle(
                &db,
                &mut artists,
                &mut genres,
                &mut albums,
                &mut songs,
                &mut song_relations,
                &mut album_relations,
                &mut seen_ids,
            )
            .await
            {
                log::error!("Flush cycle failed: {}", e);
            }

            last_flush = Instant::now();
        }

        if overdue || force_flush {
            if let Some(tx) = flush_ack.take() {
                let _ = tx.send(());
            }
        }

        if is_none && rx.is_closed() {
            break;
        }
    }
}

#[cfg(test)]
#[path = "flusher_tests.rs"]
mod tests;
