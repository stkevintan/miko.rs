use crate::models::{album_genre, artist, child, genre, queries, song_artist, song_genre};
use crate::service::Service;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, RelationTrait,
};

pub struct DirectoryWithChildren {
    pub dir: child::Model,
    pub children: Vec<child::ChildWithMetadata>,
    pub total_count: i64,
}

impl Service {
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

        Ok(crate::service::utils::create_indexed_list(
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

        let mut query = queries::song_with_metadata_query()
            .filter(child::Column::Parent.eq(&dir.id))
            .order_by_desc(child::Column::IsDir)
            .order_by_asc(child::Column::Title);

        if limit > 0 {
            query = query.offset(offset).limit(limit);
        }

        let children = query
            .into_model::<child::ChildWithMetadata>()
            .all(&self.db)
            .await?;

        Ok(DirectoryWithChildren {
            dir,
            children,
            total_count: total_count as i64,
        })
    }

    pub async fn get_genres(&self) -> Result<Vec<genre::GenreWithStats>, DbErr> {
        genre::Entity::find()
            .select_only()
            .column_as(genre::Column::Name, "value")
            .column_as(
                Expr::cust("COUNT(DISTINCT song_genres.song_id)"),
                "song_count",
            )
            .column_as(
                Expr::cust("COUNT(DISTINCT album_genres.album_id)"),
                "album_count",
            )
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
            .into_model::<genre::GenreWithStats>()
            .all(&self.db)
            .await
    }

    pub async fn get_top_songs(
        &self,
        artist_name: &str,
        count: u64,
    ) -> Result<Vec<child::ChildWithMetadata>, DbErr> {
        queries::song_with_metadata_query()
            .filter(
                child::Column::Id.in_subquery(
                    song_artist::Entity::find()
                        .select_only()
                        .column(song_artist::Column::SongId)
                        .join(JoinType::InnerJoin, song_artist::Relation::Artist.def())
                        .filter(artist::Column::Name.eq(artist_name))
                        .into_query(),
                ),
            )
            .order_by_desc(child::Column::PlayCount)
            .limit(count)
            .into_model::<child::ChildWithMetadata>()
            .all(&self.db)
            .await
    }
}
