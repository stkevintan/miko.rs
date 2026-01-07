use crate::browser::{Browser, DirectoryWithChildren, GenreWithStats, utils::strip_articles};
use crate::models::{artist, child};
use sea_orm::{
    ColumnTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Statement,
};

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
        let articles: Vec<&str> = ignored_articles.split_whitespace().collect();

        let mut index_map: std::collections::BTreeMap<String, Vec<artist::Model>> =
            std::collections::BTreeMap::new();

        for child in children {
            if child.title.is_empty() {
                continue;
            }

            let sort_name = strip_articles(&child.title, &articles);
            let first_char = sort_name
                .chars()
                .next()
                .unwrap_or(' ')
                .to_uppercase()
                .to_string();

            index_map
                .entry(first_char)
                .or_default()
                .push(artist::Model {
                    id: child.id,
                    name: child.title,
                    cover_art: "".to_string(),
                    artist_image_url: "".to_string(),
                    starred: None,
                    user_rating: 0,
                    average_rating: 0.0,
                });
        }

        let mut result: Vec<(String, Vec<artist::Model>)> = index_map.into_iter().collect();
        for (_, artists) in &mut result {
            artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }

        Ok(result)
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
                DbBackend::Sqlite,
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
        GenreWithStats::find_by_statement(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            SELECT
                g.name AS value,
                (SELECT COUNT(*) FROM song_genres sg WHERE sg.genre_name = g.name) AS song_count,
                (SELECT COUNT(*) FROM albums a WHERE a.genre = g.name) AS album_count
            FROM genres g
            ORDER BY g.name ASC
            "#,
        ))
        .all(&self.db)
        .await
    }
}
