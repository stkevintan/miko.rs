use crate::browser::types::{ChildWithMetadata, BookmarkWithMetadata};
use crate::browser::Browser;
use crate::models::{bookmark, child, play_queue, play_queue_song};
use chrono::Utc;
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set, TransactionError, TransactionTrait,
};

impl Browser {
    pub async fn get_bookmarks(&self, username: &str) -> Result<Vec<(bookmark::Model, ChildWithMetadata)>, DbErr> {
        let results = Self::song_with_metadata_query()
            .join(JoinType::InnerJoin, child::Relation::Bookmarks.def())
            .filter(bookmark::Column::Username.eq(username))
            .order_by_desc(bookmark::Column::UpdatedAt)
            .column_as(bookmark::Column::Username, "b_username")
            .column_as(bookmark::Column::SongId, "b_song_id")
            .column_as(bookmark::Column::Position, "b_position")
            .column_as(bookmark::Column::Comment, "b_comment")
            .column_as(bookmark::Column::CreatedAt, "b_created_at")
            .column_as(bookmark::Column::UpdatedAt, "b_updated_at")
            .into_model::<BookmarkWithMetadata>()
            .all(&self.db)
            .await?;

        Ok(results
            .into_iter()
            .map(|r| {
                (
                    bookmark::Model {
                        username: r.b_username,
                        song_id: r.b_song_id,
                        position: r.b_position,
                        comment: r.b_comment,
                        created_at: r.b_created_at,
                        updated_at: r.b_updated_at,
                    },
                    r.child,
                )
            })
            .collect())
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
            // play_queue_song::Column::Index represents the order of songs in the list.
            let songs = Self::song_with_metadata_query()
                .join(JoinType::InnerJoin, child::Relation::PlayQueueSongs.def())
                .filter(play_queue_song::Column::Username.eq(username))
                .order_by_asc(play_queue_song::Column::Index)
                .into_model::<ChildWithMetadata>()
                .all(&self.db)
                .await?;

            Ok(Some((queue, songs)))
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
                           let entries = song_ids.into_iter().enumerate().map(|(i, sid)| {
                            play_queue_song::ActiveModel {
                                username: Set(username.clone()),
                                song_id: Set(sid),
                                index: Set(i as i32),
                            }
                        });
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
