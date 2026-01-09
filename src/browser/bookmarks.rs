use crate::browser::types::ChildWithMetadata;
use crate::browser::Browser;
use crate::models::{bookmark, play_queue, play_queue_song};
use chrono::Utc;
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set, TransactionError, TransactionTrait,
};

impl Browser {
    pub async fn get_bookmarks(&self, username: &str) -> Result<Vec<(bookmark::Model, ChildWithMetadata)>, DbErr> {
        let bookmarks = bookmark::Entity::find()
            .filter(bookmark::Column::Username.eq(username))
            .order_by_desc(bookmark::Column::UpdatedAt)
            .all(&self.db)
            .await?;

        if bookmarks.is_empty() {
            return Ok(Vec::new());
        }

        let song_ids: Vec<String> = bookmarks.iter().map(|b| b.song_id.clone()).collect();
        let songs = self.get_songs_by_ids(&song_ids).await?;
        let song_map: std::collections::HashMap<String, ChildWithMetadata> = songs
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

        let mut result = Vec::new();
        for b in bookmarks {
            if let Some(s) = song_map.get(&b.song_id) {
                result.push((b, s.clone()));
            }
        }

        Ok(result)
    }

    pub async fn create_bookmark(
        &self,
        username: &str,
        song_id: &str,
        position: i64,
        comment: Option<String>,
    ) -> Result<(), DbErr> {
        let now = Utc::now();
        let bm = bookmark::ActiveModel {
            username: Set(username.to_string()),
            song_id: Set(song_id.to_string()),
            position: Set(position),
            comment: Set(comment),
            created_at: Set(now),
            updated_at: Set(now),
        };

        bookmark::Entity::insert(bm)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    bookmark::Column::Username,
                    bookmark::Column::SongId,
                ])
                .update_columns([
                    bookmark::Column::Position,
                    bookmark::Column::Comment,
                    bookmark::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn delete_bookmark(&self, username: &str, song_id: &str) -> Result<(), DbErr> {
        bookmark::Entity::delete_many()
            .filter(bookmark::Column::Username.eq(username))
            .filter(bookmark::Column::SongId.eq(song_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn get_play_queue(&self, username: &str) -> Result<Option<(play_queue::Model, Vec<ChildWithMetadata>)>, DbErr> {
        let queue = play_queue::Entity::find_by_id(username)
            .one(&self.db)
            .await?;

        if let Some(queue) = queue {
            let songs_with_pos = play_queue_song::Entity::find()
                .filter(play_queue_song::Column::Username.eq(username))
                .order_by_asc(play_queue_song::Column::Position)
                .all(&self.db)
                .await?;

            let song_ids: Vec<String> = songs_with_pos.iter().map(|s| s.song_id.clone()).collect();
            let songs = self.get_songs_by_ids(&song_ids).await?;
            let song_map: std::collections::HashMap<String, ChildWithMetadata> = songs
                .into_iter()
                .map(|s| (s.id.clone(), s))
                .collect();

            let mut result_songs = Vec::new();
            for s_pos in songs_with_pos {
                if let Some(s) = song_map.get(&s_pos.song_id) {
                    result_songs.push(s.clone());
                }
            }

            Ok(Some((queue, result_songs)))
        } else {
            Ok(None)
        }
    }

    pub async fn save_play_queue(
        &self,
        username: &str,
        current: Option<String>,
        position: i64,
        song_ids: Vec<String>,
        changed_by: &str,
    ) -> Result<(), DbErr> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                let username = username.to_string();
                let changed_by = changed_by.to_string();
                Box::pin(async move {
                    let now = Utc::now();
                    let pq = play_queue::ActiveModel {
                        username: Set(username.clone()),
                        current: Set(current),
                        position: Set(position),
                        changed: Set(now),
                        changed_by: Set(changed_by),
                    };

                    play_queue::Entity::insert(pq)
                        .on_conflict(
                            sea_orm::sea_query::OnConflict::column(play_queue::Column::Username)
                                .update_columns([
                                    play_queue::Column::Current,
                                    play_queue::Column::Position,
                                    play_queue::Column::Changed,
                                    play_queue::Column::ChangedBy,
                                ])
                                .to_owned(),
                        )
                        .exec(txn)
                        .await?;

                    play_queue_song::Entity::delete_many()
                        .filter(play_queue_song::Column::Username.eq(&username))
                        .exec(txn)
                        .await?;

                    if !song_ids.is_empty() {
                        let mut entries = Vec::new();
                        for (i, sid) in song_ids.into_iter().enumerate() {
                            entries.push(play_queue_song::ActiveModel {
                                username: Set(username.clone()),
                                song_id: Set(sid),
                                position: Set(i as i32),
                            });
                        }
                        play_queue_song::Entity::insert_many(entries).exec(txn).await?;
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
}
