use crate::browser::{Browser, PlaylistWithSongs, PlaylistWithStats, UpdatePlaylistOptions};
use crate::models::{child, playlist, playlist_song};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait,
    QueryFilter, QuerySelect, TransactionTrait, Set, TransactionError,
    JoinType, QueryOrder,
};
use std::collections::HashSet;
use chrono::Utc;

impl Browser {
    pub async fn create_playlist(
        &self,
        name: String,
        owner: String,
        song_ids: Vec<String>,
    ) -> Result<i32, DbErr> {
        self.db
            .transaction::<_, i32, DbErr>(|txn| {
                Box::pin(async move {
                    let now = Utc::now();
                    let p = playlist::ActiveModel {
                        name: Set(name),
                        owner: Set(owner),
                        created_at: Set(now),
                        updated_at: Set(now),
                        comment: Set(None),
                        public: Set(false),
                        ..Default::default()
                    };

                    let p = p.insert(txn).await?;

                    if !song_ids.is_empty() {
                        let mut songs = Vec::new();
                        for (i, song_id) in song_ids.into_iter().enumerate() {
                            songs.push(playlist_song::ActiveModel {
                                playlist_id: Set(p.id),
                                song_id: Set(song_id),
                                position: Set(i as i32),
                                ..Default::default()
                            });
                        }
                        playlist_song::Entity::insert_many(songs).exec(txn).await?;
                    }

                    Ok(p.id)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    pub async fn update_playlist(
        &self,
        playlist_id: i32,
        username: &str,
        opts: UpdatePlaylistOptions,
    ) -> Result<(), DbErr> {
        let username = username.to_string();
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let mut p: playlist::ActiveModel = playlist::Entity::find_by_id(playlist_id)
                        .one(txn)
                        .await?
                        .ok_or_else(|| DbErr::Custom("Playlist not found".to_string()))?
                        .into();

                    if p.owner.as_ref() != &username {
                        return Err(DbErr::Custom("Permission denied".to_string()));
                    }

                    if let Some(name) = opts.name {
                        p.name = Set(name);
                    }
                    if let Some(comment) = opts.comment {
                        p.comment = Set(Some(comment));
                    }
                    if let Some(public) = opts.public {
                        p.public = Set(public);
                    }

                    p.updated_at = Set(Utc::now());
                    p.update(txn).await?;

                    if !opts.song_ids_to_add.is_empty() || !opts.song_indices_to_remove.is_empty() {
                        let songs = playlist_song::Entity::find()
                            .filter(playlist_song::Column::PlaylistId.eq(playlist_id))
                            .order_by_asc(playlist_song::Column::Position)
                            .all(txn)
                            .await?;

                        let mut song_ids: Vec<String> = songs.into_iter().map(|s| s.song_id).collect();

                        if !opts.song_indices_to_remove.is_empty() {
                            let to_remove: HashSet<usize> = opts
                                .song_indices_to_remove
                                .into_iter()
                                .map(|i| i as usize)
                                .collect();

                            let mut new_song_ids = Vec::new();
                            for (i, sid) in song_ids.into_iter().enumerate() {
                                if !to_remove.contains(&i) {
                                    new_song_ids.push(sid);
                                }
                            }
                            song_ids = new_song_ids;
                        }

                        if !opts.song_ids_to_add.is_empty() {
                            song_ids.extend(opts.song_ids_to_add);
                        }

                        // Re-sync with database
                        playlist_song::Entity::delete_many()
                            .filter(playlist_song::Column::PlaylistId.eq(playlist_id))
                            .exec(txn)
                            .await?;

                        if !song_ids.is_empty() {
                            let new_entries: Vec<playlist_song::ActiveModel> = song_ids
                                .into_iter()
                                .enumerate()
                                .map(|(i, sid)| playlist_song::ActiveModel {
                                    playlist_id: Set(playlist_id),
                                    song_id: Set(sid),
                                    position: Set(i as i32),
                                    ..Default::default()
                                })
                                .collect();

                            playlist_song::Entity::insert_many(new_entries)
                                .exec(txn)
                                .await?;
                        }
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(e) => e,
                TransactionError::Transaction(e) => e,
            })
    }

    pub async fn delete_playlist(&self, id: i32, username: &str) -> Result<(), DbErr> {
        let p = playlist::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::Custom("Playlist not found".to_string()))?;

        if p.owner != username {
            return Err(DbErr::Custom("Permission denied".to_string()));
        }

        playlist::Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn get_playlists(
        &self,
        username: &str,
        target_username: &str,
    ) -> Result<Vec<PlaylistWithStats>, DbErr> {
        let mut query = playlist::Entity::find()
            .column_as(playlist_song::Column::Id.count(), "song_count")
            .column_as(child::Column::Duration.sum(), "duration")
            .join_rev(
                JoinType::LeftJoin,
                playlist_song::Entity::belongs_to(playlist::Entity)
                    .from(playlist_song::Column::PlaylistId)
                    .to(playlist::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                child::Entity::belongs_to(playlist_song::Entity)
                    .from(child::Column::Id)
                    .to(playlist_song::Column::SongId)
                    .into(),
            )
            .filter(playlist::Column::Owner.eq(target_username))
            .group_by(playlist::Column::Id);

        if username != target_username {
            query = query.filter(playlist::Column::Public.eq(true));
        }

        query.into_model::<PlaylistWithStats>().all(&self.db).await
    }

    pub async fn get_playlist(&self, id: i32) -> Result<Option<PlaylistWithSongs>, DbErr> {
        let playlist = playlist::Entity::find()
            .filter(playlist::Column::Id.eq(id))
            .column_as(playlist_song::Column::Id.count(), "song_count")
            .column_as(child::Column::Duration.sum(), "duration")
            .join_rev(
                JoinType::LeftJoin,
                playlist_song::Entity::belongs_to(playlist::Entity)
                    .from(playlist_song::Column::PlaylistId)
                    .to(playlist::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                child::Entity::belongs_to(playlist_song::Entity)
                    .from(child::Column::Id)
                    .to(playlist_song::Column::SongId)
                    .into(),
            )
            .group_by(playlist::Column::Id)
            .into_model::<PlaylistWithStats>()
            .one(&self.db)
            .await?;

        if let Some(playlist) = playlist {
            let songs = child::Entity::find()
                .join_rev(
                    JoinType::InnerJoin,
                    playlist_song::Entity::belongs_to(child::Entity)
                        .from(playlist_song::Column::SongId)
                        .to(child::Column::Id)
                        .into(),
                )
                .filter(playlist_song::Column::PlaylistId.eq(id))
                .order_by_asc(playlist_song::Column::Position)
                .all(&self.db)
                .await?;

            Ok(Some(PlaylistWithSongs {
                playlist,
                entry: songs,
            }))
        } else {
            Ok(None)
        }
    }
}
