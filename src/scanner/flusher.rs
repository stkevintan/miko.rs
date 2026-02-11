use crate::models::{
    album, album_artist, album_genre, artist, child, genre, lyrics, song_artist, song_genre,
};
use crate::scanner::seen;
use crate::scanner::types::UpsertMessage;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::time::{Duration, Instant};

pub async fn run_flusher(db: DatabaseConnection, mut rx: tokio::sync::mpsc::Receiver<UpsertMessage>) {
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

        // Determine if songs or relations need to be flushed
        let flush_songs = songs.len() >= 100 || (overdue && !songs.is_empty());
        let flush_song_relations = song_relations.len() >= 100 || (overdue && !song_relations.is_empty());
        let flush_album_relations = album_relations.len() >= 100 || (overdue && !album_relations.is_empty());

        // If we're about to flush songs or relations, ensure their dependencies are flushed first
        let force_deps = flush_songs || flush_song_relations || flush_album_relations;

        if artists.len() >= 100 || (overdue && !artists.is_empty()) || (force_deps && !artists.is_empty()) {
            let items = std::mem::take(&mut artists);
            if let Err(e) = artist::Entity::insert_many(items)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(artist::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(&db)
                .await
            {
                log::error!("Failed to flush artists: {}", e);
            }
        }
        if albums.len() >= 100 || (overdue && !albums.is_empty()) || (force_deps && !albums.is_empty()) {
            let items = std::mem::take(&mut albums);
            if let Err(e) = album::Entity::insert_many(items)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(album::Column::Id)
                        .update_columns([album::Column::Year])
                        .to_owned(),
                )
                .exec_without_returning(&db)
                .await
            {
                log::error!("Failed to flush albums: {}", e);
            }
        }
        if genres.len() >= 50 || (overdue && !genres.is_empty()) || (force_deps && !genres.is_empty()) {
            let items = std::mem::take(&mut genres);
            if let Err(e) = genre::Entity::insert_many(items)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(genre::Column::Name)
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(&db)
                .await
            {
                log::error!("Failed to flush genres: {}", e);
            }
        }
        if flush_songs || (force_deps && !songs.is_empty()) {
            let items = std::mem::take(&mut songs);
            if let Err(e) = child::Entity::insert_many(items)
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
                .exec_without_returning(&db)
                .await
            {
                log::error!("Failed to flush songs: {}", e);
            }
        }
        if flush_song_relations {
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
                    .filter(lyrics::Column::SongId.is_in(&song_ids))
                    .exec(&txn)
                    .await?;

                if !all_artists.is_empty() {
                    song_artist::Entity::insert_many(all_artists)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::columns([
                                song_artist::Column::SongId,
                                song_artist::Column::ArtistId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec_without_returning(&txn)
                        .await?;
                }
                if !all_genres.is_empty() {
                    song_genre::Entity::insert_many(all_genres)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::columns([
                                song_genre::Column::SongId,
                                song_genre::Column::GenreName,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec_without_returning(&txn)
                        .await?;
                }
                if !all_lyrics.is_empty() {
                    lyrics::Entity::insert_many(all_lyrics)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::column(lyrics::Column::SongId)
                                .do_nothing()
                                .to_owned(),
                        )
                        .exec_without_returning(&txn)
                        .await?;
                }

                txn.commit().await?;
                Ok::<(), sea_orm::DbErr>(())
            };

            if let Err(e) = flush_op.await {
                log::error!("Failed to flush song relations: {}", e);
            }
        }

        if flush_album_relations {
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
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::columns([
                                album_artist::Column::AlbumId,
                                album_artist::Column::ArtistId,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec_without_returning(&txn)
                        .await?;
                }
                if !all_genres.is_empty() {
                    album_genre::Entity::insert_many(all_genres)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::columns([
                                album_genre::Column::AlbumId,
                                album_genre::Column::GenreName,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec_without_returning(&txn)
                        .await?;
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
            last_flush = Instant::now();
            if let Some(tx) = flush_ack.take() {
                let _ = tx.send(());
            }
        }

        if is_none && rx.is_closed() {
            break;
        }
    }
}
