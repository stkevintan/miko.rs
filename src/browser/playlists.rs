use crate::browser::{Browser, PlaylistWithSongs, PlaylistWithStats, UpdatePlaylistOptions};
use crate::models::{child, playlist, playlist_song};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult,
    QueryFilter, QuerySelect, Statement, TransactionTrait, Set, TransactionError,
};
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
                        comment: Set("".to_string()),
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
                        p.comment = Set(comment);
                    }
                    if let Some(public) = opts.public {
                        p.public = Set(public);
                    }

                    p.updated_at = Set(Utc::now());
                    p.update(txn).await?;

                    if !opts.song_ids_to_add.is_empty() {
                        let max_pos: Option<i32> = playlist_song::Entity::find()
                            .filter(playlist_song::Column::PlaylistId.eq(playlist_id))
                            .select_only()
                            .column_as(playlist_song::Column::Position.max(), "max_pos")
                            .into_tuple()
                            .one(txn)
                            .await?
                            .unwrap_or(None);

                        let start_pos = max_pos.map(|p| p + 1).unwrap_or(0);
                        let mut songs_to_add = Vec::new();
                        for (i, song_id) in opts.song_ids_to_add.into_iter().enumerate() {
                            songs_to_add.push(playlist_song::ActiveModel {
                                playlist_id: Set(playlist_id),
                                song_id: Set(song_id),
                                position: Set(start_pos + i as i32),
                                ..Default::default()
                            });
                        }
                        playlist_song::Entity::insert_many(songs_to_add)
                            .exec(txn)
                            .await?;
                    }

                    if !opts.song_indices_to_remove.is_empty() {
                        playlist_song::Entity::delete_many()
                            .filter(playlist_song::Column::PlaylistId.eq(playlist_id))
                            .filter(playlist_song::Column::Position.is_in(opts.song_indices_to_remove))
                            .exec(txn)
                            .await?;

                        // Re-index
                        let sql = r#"
                            UPDATE playlist_songs
                            SET position = (
                                SELECT COUNT(*)
                                FROM playlist_songs AS ps
                                WHERE ps.playlist_id = playlist_songs.playlist_id
                                AND ps.position < playlist_songs.position
                            )
                            WHERE playlist_id = ?
                        "#;

                        txn.execute(Statement::from_sql_and_values(
                            DbBackend::Sqlite,
                            sql,
                            vec![playlist_id.into()],
                        ))
                        .await?;
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
        let mut sql = "SELECT playlists.*, \
                       COUNT(playlist_songs.id) as song_count, \
                       CAST(COALESCE(SUM(children.duration), 0) AS INTEGER) as duration \
                       FROM playlists \
                       LEFT JOIN playlist_songs ON playlist_songs.playlist_id = playlists.id \
                       LEFT JOIN children ON children.id = playlist_songs.song_id \
                       WHERE playlists.owner = ?".to_string();

        if username != target_username {
            sql.push_str(" AND playlists.public = 1");
        }

        sql.push_str(" GROUP BY playlists.id");

        PlaylistWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            vec![target_username.into()],
        ))
        .all(&self.db)
        .await
    }

    pub async fn get_playlist(&self, id: i32) -> Result<Option<PlaylistWithSongs>, DbErr> {
        let playlist = PlaylistWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "SELECT playlists.*, \
             COUNT(playlist_songs.id) as song_count, \
             CAST(COALESCE(SUM(children.duration), 0) AS INTEGER) as duration \
             FROM playlists \
             LEFT JOIN playlist_songs ON playlist_songs.playlist_id = playlists.id \
             LEFT JOIN children ON children.id = playlist_songs.song_id \
             WHERE playlists.id = ? \
             GROUP BY playlists.id",
            vec![id.into()],
        ))
        .one(&self.db)
        .await?;

        if let Some(playlist) = playlist {
            let songs = child::Entity::find()
                .from_raw_sql(Statement::from_sql_and_values(
                    DbBackend::Sqlite,
                    "SELECT children.* FROM children \
                     JOIN playlist_songs ON playlist_songs.song_id = children.id \
                     WHERE playlist_songs.playlist_id = ? \
                     ORDER BY playlist_songs.position ASC",
                    vec![id.into()],
                ))
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
