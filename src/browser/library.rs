use crate::browser::{AlbumListOptions, AlbumWithStats, ArtistWithStats, Browser, utils::strip_articles};
use crate::models::{child};
use sea_orm::{
    ColumnTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, Statement,
};

impl Browser {
    pub async fn get_albums(
        &self,
        opts: AlbumListOptions,
    ) -> Result<Vec<AlbumWithStats>, sea_orm::DbErr> {
        let list_type = opts.r#type.as_deref().unwrap_or("newest");
        let size = opts.size.unwrap_or(10);
        let offset = opts.offset.unwrap_or(0);

        let mut sql = "SELECT albums.*, COALESCE(stats.song_count, 0) AS song_count, \
                       CAST(COALESCE(stats.duration, 0) AS INTEGER) AS duration, \
                       CAST(COALESCE(stats.play_count, 0) AS INTEGER) AS play_count \
                       FROM albums \
                       LEFT JOIN (SELECT album_id, COUNT(*) as song_count, SUM(duration) as duration, SUM(play_count) as play_count FROM children WHERE is_dir = 0 GROUP BY album_id) stats \
                       ON stats.album_id = albums.id".to_string();

        let mut conditions = Vec::new();
        if let Some(folder_id) = opts.music_folder_id {
            sql = format!("{} JOIN children c2 ON c2.album_id = albums.id", sql);
            conditions.push(format!("c2.music_folder_id = {}", folder_id));
        }

        match list_type {
            "byYear" => {
                if let Some(from) = opts.from_year {
                    conditions.push(format!("albums.year >= {}", from));
                }
                if let Some(to) = opts.to_year {
                    conditions.push(format!("albums.year <= {}", to));
                }
            }
            "byGenre" => {
                if let Some(ref genre) = opts.genre {
                    conditions.push(format!("albums.genre = '{}'", genre.replace("'", "''")));
                }
            }
            "starred" => {
                conditions.push("albums.starred IS NOT NULL".to_string());
            }
            "recent" => {
                sql = format!(
                    "{}, (SELECT last_played FROM children WHERE album_id = albums.id AND is_dir = 0 ORDER BY last_played DESC LIMIT 1) AS last_played {}",
                    &sql[..sql.find(" FROM").unwrap()],
                    &sql[sql.find(" FROM").unwrap()..]
                );
                conditions.push("last_played IS NOT NULL".to_string());
            }
            _ => {}
        }

        if !conditions.is_empty() {
            sql = format!("{} WHERE {}", sql, conditions.join(" AND "));
        }

        if opts.music_folder_id.is_some() {
            sql = format!("{} GROUP BY albums.id", sql);
        }

        let sql = match list_type {
            "random" => format!("{} ORDER BY RANDOM()", sql),
            "newest" => format!("{} ORDER BY albums.created DESC", sql),
            "frequent" => format!("{} ORDER BY play_count DESC", sql),
            "recent" => format!("{} ORDER BY last_played DESC", sql),
            "starred" => format!("{} ORDER BY albums.starred DESC", sql),
            "alphabeticalByName" => format!("{} ORDER BY albums.name ASC", sql),
            "alphabeticalByArtist" => format!("{} ORDER BY albums.artist ASC", sql),
            "byYear" => format!("{} ORDER BY albums.year DESC", sql),
            _ => format!("{} ORDER BY albums.created DESC", sql),
        };

        let sql = format!("{} LIMIT {} OFFSET {}", sql, size, offset);

        AlbumWithStats::find_by_statement(Statement::from_string(DbBackend::Sqlite, sql))
            .all(&self.db)
            .await
    }

    pub async fn get_artists(&self, ignored_articles: &str) -> Result<Vec<(String, Vec<ArtistWithStats>)>, DbErr> {
        let artists = ArtistWithStats::find_by_statement(Statement::from_string(
            DbBackend::Sqlite,
            r#"
            SELECT a.*, COALESCE(stats.album_count, 0) AS album_count
            FROM artists a
            LEFT JOIN (SELECT artist_id, COUNT(*) AS album_count FROM albums GROUP BY artist_id) stats 
            ON stats.artist_id = a.id
            "#
        ))
        .all(&self.db)
        .await?;

        let articles: Vec<&str> = ignored_articles.split_whitespace().collect();
        let mut index_map: std::collections::BTreeMap<String, Vec<ArtistWithStats>> = std::collections::BTreeMap::new();

        for artist in artists {
            if artist.name.is_empty() {
                continue;
            }

            let sort_name = strip_articles(&artist.name, &articles);
            let first_char = sort_name.chars().next().unwrap_or(' ').to_uppercase().to_string();

            index_map.entry(first_char).or_default().push(artist);
        }

        let mut result: Vec<(String, Vec<ArtistWithStats>)> = index_map.into_iter().collect();
        for (_, artists) in &mut result {
            artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }

        Ok(result)
    }

    pub async fn get_artist(&self, id: &str) -> Result<(ArtistWithStats, Vec<AlbumWithStats>), DbErr> {
        let artist = ArtistWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            r#"
            SELECT a.*, COALESCE(stats.album_count, 0) AS album_count
            FROM artists a
            LEFT JOIN (SELECT artist_id, COUNT(*) AS album_count FROM albums GROUP BY artist_id) stats 
            ON stats.artist_id = a.id
            WHERE a.id = ?
            "#,
            vec![id.into()],
        ))
        .one(&self.db)
        .await?
        .ok_or(DbErr::RecordNotFound("Artist not found".into()))?;

        let albums = self.get_albums_by_artist(id).await?;

        Ok((artist, albums))
    }

    pub async fn get_albums_by_artist(&self, artist_id: &str) -> Result<Vec<AlbumWithStats>, DbErr> {
        let sql = "SELECT albums.*, COALESCE(stats.song_count, 0) AS song_count, \
                       CAST(COALESCE(stats.duration, 0) AS INTEGER) AS duration, \
                       CAST(COALESCE(stats.play_count, 0) AS INTEGER) AS play_count \
                       FROM albums \
                       LEFT JOIN (SELECT album_id, COUNT(*) as song_count, SUM(duration) as duration, SUM(play_count) as play_count FROM children WHERE is_dir = 0 GROUP BY album_id) stats \
                       ON stats.album_id = albums.id \
                       WHERE albums.artist_id = ? \
                       ORDER BY albums.year DESC, albums.name ASC".to_string();

        AlbumWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            vec![artist_id.into()],
        ))
        .all(&self.db)
        .await
    }

    pub async fn get_album(&self, id: &str) -> Result<(AlbumWithStats, Vec<child::Model>), DbErr> {
        let album = self.get_album_with_stats(id).await?;

        let songs = child::Entity::find()
            .filter(child::Column::AlbumId.eq(id))
            .filter(child::Column::IsDir.eq(false))
            .order_by_asc(child::Column::DiscNumber)
            .order_by_asc(child::Column::Track)
            .all(&self.db)
            .await?;

        Ok((album, songs))
    }

    async fn get_album_with_stats(&self, id: &str) -> Result<AlbumWithStats, DbErr> {
        let sql = "SELECT albums.*, COALESCE(stats.song_count, 0) AS song_count, \
                       CAST(COALESCE(stats.duration, 0) AS INTEGER) AS duration, \
                       CAST(COALESCE(stats.play_count, 0) AS INTEGER) AS play_count \
                       FROM albums \
                       LEFT JOIN (SELECT album_id, COUNT(*) as song_count, SUM(duration) as duration, SUM(play_count) as play_count FROM children WHERE is_dir = 0 GROUP BY album_id) stats \
                       ON stats.album_id = albums.id \
                       WHERE albums.id = ?".to_string();

        AlbumWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &sql,
            vec![id.into()],
        ))
        .one(&self.db)
        .await?
        .ok_or(DbErr::RecordNotFound("Album not found".into()))
    }

    pub async fn get_song(&self, id: &str) -> Result<child::Model, DbErr> {
        child::Entity::find_by_id(id)
            .filter(child::Column::IsDir.eq(false))
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Song not found".into()))
    }

    pub async fn get_random_songs(
        &self,
        opts: AlbumListOptions,
    ) -> Result<Vec<child::Model>, sea_orm::DbErr> {
        let size = opts.size.unwrap_or(10);

        let mut sql = "SELECT * FROM children WHERE is_dir = 0".to_string();
        let mut conditions = Vec::new();

        if let Some(folder_id) = opts.music_folder_id {
            conditions.push(format!("music_folder_id = {}", folder_id));
        }

        if let Some(ref genre) = opts.genre {
            conditions.push(format!("genre = '{}'", genre.replace("'", "''")));
        }

        if let Some(from) = opts.from_year {
            conditions.push(format!("year >= {}", from));
        }

        if let Some(to) = opts.to_year {
            conditions.push(format!("year <= {}", to));
        }

        if !conditions.is_empty() {
            sql = format!("{} AND {}", sql, conditions.join(" AND "));
        }

        let sql = format!("{} ORDER BY RANDOM() LIMIT {}", sql, size);

        child::Entity::find()
            .from_raw_sql(Statement::from_string(DbBackend::Sqlite, sql))
            .all(&self.db)
            .await
    }

    pub async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: u64,
        offset: u64,
        folder_id: Option<i32>,
    ) -> Result<Vec<child::Model>, sea_orm::DbErr> {
        let mut db_query = child::Entity::find()
            .filter(child::Column::IsDir.eq(false))
            .filter(child::Column::Genre.eq(genre));

        if let Some(f_id) = folder_id {
            db_query = db_query.filter(child::Column::MusicFolderId.eq(f_id));
        }

        db_query
            .limit(count)
            .offset(offset)
            .all(&self.db)
            .await
    }

    pub async fn get_starred_items(
        &self,
        folder_id: Option<i32>,
    ) -> Result<(Vec<ArtistWithStats>, Vec<AlbumWithStats>, Vec<child::Model>), sea_orm::DbErr> {
        // Artists
        let mut artist_sql = "SELECT a.*, COALESCE(stats.album_count, 0) AS album_count \
                             FROM artists a \
                             LEFT JOIN (SELECT artist_id, COUNT(*) AS album_count FROM albums GROUP BY artist_id) stats \
                             ON stats.artist_id = a.id \
                             WHERE a.starred IS NOT NULL".to_string();

        if let Some(f_id) = folder_id {
            artist_sql = format!("{} AND EXISTS (SELECT 1 FROM children c WHERE c.artist_id = a.id AND c.music_folder_id = {})", artist_sql, f_id);
        }

        let artist_sql = format!("{} ORDER BY a.starred DESC", artist_sql);

        let artists = ArtistWithStats::find_by_statement(Statement::from_string(DbBackend::Sqlite, artist_sql))
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
        let mut song_query = child::Entity::find()
            .filter(child::Column::IsDir.eq(false))
            .filter(child::Column::Starred.is_not_null());

        if let Some(f_id) = folder_id {
            song_query = song_query.filter(child::Column::MusicFolderId.eq(f_id));
        }
        let songs = song_query.all(&self.db).await?;

        Ok((artists, albums, songs))
    }
}
