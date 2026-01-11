use crate::browser::types::{AlbumListOptions, ArtistWithStats};
use crate::models::queries::{self, AlbumWithStats, ChildWithMetadata};
use crate::browser::Browser;
use crate::models::{album, album_artist, artist, child, genre, song_artist, album_genre};
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, JoinType, Order, QueryFilter, QueryOrder, QuerySelect,
};
use sea_orm::sea_query::{Expr, ExprTrait, Query};

impl Browser {
    pub async fn get_albums(
        &self,
        opts: AlbumListOptions,
    ) -> Result<Vec<AlbumWithStats>, sea_orm::DbErr> {
        let list_type = opts.r#type.as_deref().unwrap_or("newest");
        let size = opts.size.unwrap_or(10);
        let offset = opts.offset.unwrap_or(0);

        let mut query = queries::album_with_stats_query();

        if let Some(folder_id) = opts.music_folder_id {
            query = query.filter(child::Column::MusicFolderId.eq(folder_id));
        }

        match list_type {
            "byYear" => {
                if let Some(from) = opts.from_year {
                    query = query.filter(album::Column::Year.gte(from));
                }
                if let Some(to) = opts.to_year {
                    query = query.filter(album::Column::Year.lte(to));
                }
            }
            "byGenre" => {
                if let Some(ref genre) = opts.genre {
                    query = query.filter(
                        album::Column::Id.in_subquery(
                            Query::select()
                                .column(album_genre::Column::AlbumId)
                                .from(album_genre::Entity)
                                .and_where(album_genre::Column::GenreName.eq(genre))
                                .to_owned()
                        )
                    );
                }
            }
            "starred" => {
                query = query.filter(album::Column::Starred.is_not_null());
            }
            "recent" => {
                query = query.having(child::Column::LastPlayed.max().is_not_null());
            }
            _ => {}
        }

        query = match list_type {
            "random" => query.order_by(Expr::cust("RANDOM()"), Order::Asc),
            "newest" => query.order_by_desc(album::Column::Created),
            "frequent" => query.order_by_desc(Expr::cust("play_count")),
            "recent" => query.order_by_desc(Expr::cust("last_played")),
            "starred" => query.order_by_desc(album::Column::Starred),
            "alphabeticalByName" => query.order_by_asc(album::Column::Name),
            "alphabeticalByArtist" => query.order_by_asc(artist::Column::Name),
            "byYear" => query.order_by_desc(album::Column::Year),
            _ => query.order_by_desc(album::Column::Created),
        };

        query
            .limit(size)
            .offset(offset)
            .into_model::<AlbumWithStats>()
            .all(&self.db)
            .await
    }

    pub async fn get_artists(&self, ignored_articles: &str) -> Result<Vec<(String, Vec<ArtistWithStats>)>, DbErr> {
        let artists = artist::Entity::find()
            .column_as(album_artist::Column::AlbumId.count(), "album_count")
            .join_rev(
                JoinType::LeftJoin,
                album_artist::Entity::belongs_to(artist::Entity)
                    .from(album_artist::Column::ArtistId)
                    .to(artist::Column::Id)
                    .into(),
            )
            .group_by(artist::Column::Id)
            .into_model::<ArtistWithStats>()
            .all(&self.db)
            .await?;

        Ok(crate::browser::utils::create_indexed_list(
            artists,
            ignored_articles,
            |a| &a.name,
        ))
    }

    pub async fn get_artist(&self, id: &str) -> Result<(ArtistWithStats, Vec<AlbumWithStats>), DbErr> {
        let artist = artist::Entity::find()
            .filter(artist::Column::Id.eq(id))
            .column_as(album_artist::Column::AlbumId.count(), "album_count")
            .join_rev(
                JoinType::LeftJoin,
                album_artist::Entity::belongs_to(artist::Entity)
                    .from(album_artist::Column::ArtistId)
                    .to(artist::Column::Id)
                    .into(),
            )
            .group_by(artist::Column::Id)
            .into_model::<ArtistWithStats>()
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Artist not found".into()))?;

        let albums = self.get_albums_by_artist(id).await?;

        Ok((artist, albums))
    }

    pub async fn get_albums_by_artist(&self, artist_id: &str) -> Result<Vec<AlbumWithStats>, DbErr> {
        queries::album_with_stats_query()
            .join_rev(
                JoinType::InnerJoin,
                album_artist::Entity::belongs_to(album::Entity)
                    .from(album_artist::Column::AlbumId)
                    .to(album::Column::Id)
                    .into(),
            )
            .filter(album_artist::Column::ArtistId.eq(artist_id))
            .order_by_desc(album::Column::Year)
            .order_by_asc(album::Column::Name)
            .into_model::<AlbumWithStats>()
            .all(&self.db)
            .await
    }

    pub async fn get_album(&self, id: &str) -> Result<(AlbumWithStats, Vec<ChildWithMetadata>), DbErr> {
        let album = self.get_album_with_stats(id).await?;

        let songs = queries::song_with_metadata_query()
            .filter(child::Column::AlbumId.eq(id))
            .filter(child::Column::IsDir.eq(false))
            .order_by_asc(child::Column::DiscNumber)
            .order_by_asc(child::Column::Track)
            .into_model::<ChildWithMetadata>()
            .all(&self.db)
            .await?;

        Ok((album, songs))
    }

    async fn get_album_with_stats(&self, id: &str) -> Result<AlbumWithStats, DbErr> {
        queries::album_with_stats_query()
            .filter(album::Column::Id.eq(id))
            .into_model::<AlbumWithStats>()
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Album not found".into()))
    }

    pub async fn get_song(&self, id: &str) -> Result<ChildWithMetadata, DbErr> {
        queries::song_with_metadata_query()
            .filter(child::Column::Id.eq(id))
            .filter(child::Column::IsDir.eq(false))
            .into_model::<ChildWithMetadata>()
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Song not found".into()))
    }

    pub async fn get_random_songs(
        &self,
        opts: AlbumListOptions,
    ) -> Result<Vec<ChildWithMetadata>, sea_orm::DbErr> {
        let size = opts.size.unwrap_or(10);

        let mut query = queries::song_with_metadata_query()
            .filter(child::Column::IsDir.eq(false));

        if let Some(folder_id) = opts.music_folder_id {
            query = query.filter(child::Column::MusicFolderId.eq(folder_id));
        }

        if let Some(ref g_name) = opts.genre {
            query = query.filter(genre::Column::Name.eq(g_name));
        }

        if let Some(from) = opts.from_year {
            query = query.filter(child::Column::Year.gte(from));
        }

        if let Some(to) = opts.to_year {
            query = query.filter(child::Column::Year.lte(to));
        }

        query
            .order_by(Expr::cust("RANDOM()"), Order::Asc)
            .limit(size)
            .into_model::<ChildWithMetadata>()
            .all(&self.db)
            .await
    }

    pub async fn get_songs_by_genre(
        &self,
        genre_name: &str,
        count: u64,
        offset: u64,
        folder_id: Option<i32>,
    ) -> Result<Vec<ChildWithMetadata>, sea_orm::DbErr> {
        let mut db_query = queries::song_with_metadata_query()
            .filter(child::Column::IsDir.eq(false))
            .filter(genre::Column::Name.eq(genre_name));

        if let Some(f_id) = folder_id {
            db_query = db_query.filter(child::Column::MusicFolderId.eq(f_id));
        }

        db_query
            .limit(count)
            .offset(offset)
            .into_model::<ChildWithMetadata>()
            .all(&self.db)
            .await
    }

    pub async fn get_songs_by_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<ChildWithMetadata>, sea_orm::DbErr> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        queries::song_with_metadata_query()
            .filter(child::Column::Id.is_in(ids))
            .into_model::<ChildWithMetadata>()
            .all(&self.db)
            .await
    }

    pub async fn get_starred_items(
        &self,
        folder_id: Option<i32>,
    ) -> Result<(Vec<ArtistWithStats>, Vec<AlbumWithStats>, Vec<ChildWithMetadata>), sea_orm::DbErr> {
        // Artists
        let mut artist_query = artist::Entity::find()
            .column_as(album_artist::Column::AlbumId.count(), "album_count")
            .join_rev(
                JoinType::LeftJoin,
                album_artist::Entity::belongs_to(artist::Entity)
                    .from(album_artist::Column::ArtistId)
                    .to(artist::Column::Id)
                    .into(),
            )
            .filter(artist::Column::Starred.is_not_null())
            .group_by(artist::Column::Id)
            .order_by_desc(artist::Column::Starred);

        if let Some(f_id) = folder_id {
            artist_query = artist_query.filter(
                artist::Column::Id.in_subquery(
                    Query::select()
                        .column(song_artist::Column::ArtistId)
                        .from(song_artist::Entity)
                        .join(
                            JoinType::InnerJoin,
                            child::Entity,
                            Expr::col(child::Column::Id).eq(Expr::col(song_artist::Column::SongId)),
                        )
                        .and_where(child::Column::MusicFolderId.eq(f_id))
                        .to_owned(),
                ),
            );
        }

        let artists = artist_query
            .into_model::<ArtistWithStats>()
            .all(&self.db)
            .await?;

        // Albums
        let albums = self.get_albums(AlbumListOptions {
            r#type: Some("starred".to_string()),
            size: Some(100000),
            music_folder_id: folder_id,
            ..Default::default()
        })
        .await?;

        // Songs
        let mut song_query = queries::song_with_metadata_query()
            .filter(child::Column::IsDir.eq(false))
            .filter(child::Column::Starred.is_not_null());

        if let Some(f_id) = folder_id {
            song_query = song_query.filter(child::Column::MusicFolderId.eq(f_id));
        }
        let songs = song_query.into_model::<ChildWithMetadata>().all(&self.db).await?;

        Ok((artists, albums, songs))
    }
}
