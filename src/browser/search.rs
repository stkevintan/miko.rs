use crate::browser::{AlbumWithStats, ArtistWithStats, Browser, SearchOptions};
use crate::models::{child};
use sea_orm::{
    ColumnTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QuerySelect, Statement,
};

impl Browser {
    pub async fn search(&self, opts: SearchOptions) -> Result<(Vec<ArtistWithStats>, Vec<AlbumWithStats>, Vec<child::Model>), DbErr> {
        let search_query = format!("%{}%", opts.query);
        
        // Artists
        let mut artist_sql = "SELECT a.*, COALESCE(stats.album_count, 0) AS album_count \
                             FROM artists a \
                             LEFT JOIN (SELECT artist_id, COUNT(*) AS album_count FROM albums GROUP BY artist_id) stats \
                             ON stats.artist_id = a.id \
                             WHERE a.name LIKE ?".to_string();
        
        let mut album_sql = "SELECT albums.*, COALESCE(stats.song_count, 0) AS song_count, \
                            CAST(COALESCE(stats.duration, 0) AS INTEGER) AS duration, \
                            CAST(COALESCE(stats.play_count, 0) AS INTEGER) AS play_count \
                            FROM albums \
                            LEFT JOIN (SELECT album_id, COUNT(*) as song_count, SUM(duration) as duration, SUM(play_count) as play_count FROM children WHERE is_dir = 0 GROUP BY album_id) stats \
                            ON stats.album_id = albums.id \
                            WHERE albums.name LIKE ?".to_string();

        let mut song_query = child::Entity::find()
            .filter(child::Column::IsDir.eq(false))
            .filter(
                child::Column::Title.like(&search_query)
                    .or(child::Column::Album.like(&search_query))
                    .or(child::Column::Artist.like(&search_query))
            );

        if let Some(folder_id) = opts.music_folder_id {
            artist_sql = format!("{} AND EXISTS (SELECT 1 FROM children c WHERE c.artist_id = a.id AND c.music_folder_id = {})", artist_sql, folder_id);
            album_sql = format!("{} AND EXISTS (SELECT 1 FROM children c WHERE c.album_id = albums.id AND c.music_folder_id = {})", album_sql, folder_id);
            song_query = song_query.filter(child::Column::MusicFolderId.eq(folder_id));
        }

        let artist_sql = format!("{} LIMIT {} OFFSET {}", artist_sql, opts.artist_count, opts.artist_offset);
        let album_sql = format!("{} LIMIT {} OFFSET {}", album_sql, opts.album_count, opts.album_offset);
        
        let artists = ArtistWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &artist_sql,
            vec![search_query.clone().into()],
        ))
        .all(&self.db)
        .await?;
        
        let albums = AlbumWithStats::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            &album_sql,
            vec![search_query.clone().into()],
        ))
        .all(&self.db)
        .await?;

        let songs = song_query
            .limit(opts.song_count)
            .offset(opts.song_offset)
            .all(&self.db)
            .await?;

        Ok((artists, albums, songs))
    }

    pub async fn search_songs(&self, query: &str, count: u64, offset: u64) -> Result<(Vec<child::Model>, u64), DbErr> {
        let search_query = format!("%{}%", query);
        
        let q = child::Entity::find()
            .filter(child::Column::IsDir.eq(false))
            .filter(
                child::Column::Title.like(&search_query)
                    .or(child::Column::Album.like(&search_query))
                    .or(child::Column::Artist.like(&search_query))
            );

        let total = q.clone().count(&self.db).await?;
        let songs = q.limit(count).offset(offset).all(&self.db).await?;

        Ok((songs, total))
    }
}
