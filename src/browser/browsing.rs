use crate::browser::{Browser, DirectoryWithChildren, GenreWithStats};
use crate::models::{artist, child, genre, song_genre, album};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect
};
use sea_orm::sea_query::Expr;

impl Browser {
    pub async fn get_indexes(
        &self,
        folder_id: Option<i32>,
        ignored_articles: &str,
    ) -> Result<Vec<(String, Vec<artist::Model>)>, DbErr> {
        let mut query = child::Entity::find()
            .filter(child::Column::IsDir.eq(true))
            .filter(child::Column::Parent.eq(""));

        if let Some(f_id) = folder_id {
            query = query.filter(child::Column::MusicFolderId.eq(f_id));
        }

        let children = query.all(&self.db).await?;

        let artists: Vec<artist::Model> = children
            .into_iter()
            .map(|child| artist::Model {
                id: child.id,
                name: child.title,
                cover_art: "".to_string(),
                artist_image_url: "".to_string(),
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
            .filter(child::Column::Parent.eq(dir.id.clone()))
            .count(&self.db)
            .await?;

        let mut query = child::Entity::find()
            .filter(child::Column::Parent.eq(dir.id.clone()))
            .order_by_desc(child::Column::IsDir)
            .order_by_asc(child::Column::Title);

        if limit > 0 {
            query = query.offset(offset).limit(limit);
        }

        let children = query.all(&self.db).await?;

        let mut parents = Vec::new();
        if !dir.parent.is_empty() {
            // Recursive CTE for ancestors
            let ancestors = child::Model::find_by_statement(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                r#"
                WITH RECURSIVE ancestors AS (
                    SELECT * FROM children WHERE id = ?
                    UNION ALL
                    SELECT c.* FROM children c
                    JOIN ancestors a ON c.id = a.parent
                )
                SELECT * FROM ancestors
            "#,
                vec![dir.parent.clone().into()],
            ))
            .all(&self.db)
            .await?;

            parents = ancestors;
            parents.reverse();
        }

        Ok(DirectoryWithChildren {
            dir,
            children,
            total_count: total_count as i64,
            parents,
        })
    }

    pub async fn get_genres(&self) -> Result<Vec<GenreWithStats>, DbErr> {
        genre::Entity::find()
            .select_only()
            .column_as(genre::Column::Name, "value")
            .column_as(Expr::cust("COUNT(DISTINCT song_genres.song_id)"), "song_count")
            .column_as(Expr::cust("COUNT(DISTINCT albums.id)"), "album_count")
            .join_rev(
                JoinType::LeftJoin,
                song_genre::Entity::belongs_to(genre::Entity)
                    .from(song_genre::Column::GenreName)
                    .to(genre::Column::Name)
                    .into(),
            )
            .join_rev(
                JoinType::LeftJoin,
                album::Entity::belongs_to(genre::Entity)
                    .from(album::Column::Genre)
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
