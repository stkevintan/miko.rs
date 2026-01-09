use crate::browser::{Browser, ChildWithMetadata, DirectoryWithChildren, GenreWithStats};
use crate::models::{artist, child, genre, song_genre, album, song_artist, album_artist, album_genre};
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait
};
use sea_orm::sea_query::Expr;

impl Browser {
    pub fn song_with_metadata_query() -> sea_orm::Select<child::Entity> {
        child::Entity::find()
            .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name)"), "artist")
            .column_as(Expr::cust("MIN(artists.id)"), "artist_id")
            .column_as(Expr::cust("GROUP_CONCAT(DISTINCT genres.name)"), "genre")
            .column_as(album::Column::Name, "album")
            .join_rev(
                JoinType::LeftJoin,
                song_artist::Entity::belongs_to(child::Entity)
                    .from(song_artist::Column::SongId)
                    .to(child::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                artist::Entity::belongs_to(song_artist::Entity)
                    .from(artist::Column::Id)
                    .to(song_artist::Column::ArtistId)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                song_genre::Entity::belongs_to(child::Entity)
                    .from(song_genre::Column::SongId)
                    .to(child::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                genre::Entity::belongs_to(song_genre::Entity)
                    .from(genre::Column::Name)
                    .to(song_genre::Column::GenreName)
                    .into(),
            )
            .join(JoinType::LeftJoin, child::Relation::Album.def())
            .group_by(child::Column::Id)
    }

    pub fn album_with_stats_query() -> sea_orm::Select<album::Entity> {
        album::Entity::find()
            .column_as(child::Column::Id.count(), "song_count")
            .column_as(child::Column::Duration.sum(), "duration")
            .column_as(child::Column::PlayCount.sum(), "play_count")
            .column_as(child::Column::LastPlayed.max(), "last_played")
            .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name)"), "artist")
            .column_as(Expr::cust("MIN(artists.id)"), "artist_id")
            .column_as(Expr::cust("GROUP_CONCAT(DISTINCT album_genres.genre_name)"), "genre")
            .join_rev(
                JoinType::LeftJoin,
                child::Entity::belongs_to(album::Entity)
                    .from(child::Column::AlbumId)
                    .to(album::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                album_artist::Entity::belongs_to(album::Entity)
                    .from(album_artist::Column::AlbumId)
                    .to(album::Column::Id)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                artist::Entity::belongs_to(album_artist::Entity)
                    .from(artist::Column::Id)
                    .to(album_artist::Column::ArtistId)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                album_genre::Entity::belongs_to(album::Entity)
                    .from(album_genre::Column::AlbumId)
                    .to(album::Column::Id)
                    .into(),
            )
            .group_by(album::Column::Id)
    }

    pub async fn get_indexes(
        &self,
        folder_id: Option<i32>,
        ignored_articles: &str,
    ) -> Result<Vec<(String, Vec<artist::Model>)>, DbErr> {
        let mut query = child::Entity::find()
            .filter(child::Column::IsDir.eq(true))
            .filter(child::Column::Parent.is_null());

        if let Some(f_id) = folder_id {
            query = query.filter(child::Column::MusicFolderId.eq(f_id));
        }

        let children = query.all(&self.db).await?;

        let artists: Vec<artist::Model> = children
            .into_iter()
            .map(|child| artist::Model {
                id: child.id,
                name: child.title,
                artist_image_url: None,
                starred: None,
                user_rating: 0,
                average_rating: 0.0,
            })
            .collect();

        Ok(crate::browser::utils::create_indexed_list(
            artists,
            ignored_articles,
            |a| &a.name,
        ))
    }

    pub async fn get_directory(
        &self,
        id: &str,
        offset: u64,
        limit: u64,
    ) -> Result<DirectoryWithChildren, DbErr> {
        let dir = child::Entity::find_by_id(id)
            .filter(child::Column::IsDir.eq(true))
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Directory not found".to_string()))?;

        let total_count = child::Entity::find()
            .filter(child::Column::Parent.eq(&dir.id))
            .count(&self.db)
            .await?;

        let mut query = Self::song_with_metadata_query()
            .filter(child::Column::Parent.eq(&dir.id))
            .order_by_desc(child::Column::IsDir)
            .order_by_asc(child::Column::Title);

        if limit > 0 {
            query = query.offset(offset).limit(limit);
        }

        let children = query.into_model::<ChildWithMetadata>().all(&self.db).await?;

        Ok(DirectoryWithChildren {
            dir,
            children,
            total_count: total_count as i64,
        })
    }

    pub async fn get_genres(&self) -> Result<Vec<GenreWithStats>, DbErr> {
        genre::Entity::find()
            .select_only()
            .column_as(genre::Column::Name, "value")
            .column_as(Expr::cust("COUNT(DISTINCT song_genres.song_id)"), "song_count")
            .column_as(Expr::cust("COUNT(DISTINCT album_genres.album_id)"), "album_count")
            .join_rev(
                JoinType::LeftJoin,
                song_genre::Entity::belongs_to(genre::Entity)
                    .from(song_genre::Column::GenreName)
                    .to(genre::Column::Name)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                album_genre::Entity::belongs_to(genre::Entity)
                    .from(album_genre::Column::GenreName)
                    .to(genre::Column::Name)
                    .into(),
            )
            .group_by(genre::Column::Name)
            .order_by_asc(genre::Column::Name)
            .into_model::<GenreWithStats>()
            .all(&self.db)
            .await
    }
}
