use crate::browser::{AlbumWithStats, ArtistWithStats, Browser, SearchOptions};
use crate::models::{album, album_artist, artist, child, song_artist};
use sea_orm::{
    ColumnTrait, DbErr, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QuerySelect,
};
use sea_orm::sea_query::{Expr, Query};

impl Browser {
    pub async fn search(&self, opts: SearchOptions) -> Result<(Vec<ArtistWithStats>, Vec<AlbumWithStats>, Vec<child::Model>), DbErr> {
        let clean_query = opts.query.trim().trim_matches('"');
        let search_query = format!("%{}%", clean_query);
        
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
            .filter(artist::Column::Name.like(&search_query))
            .group_by(artist::Column::Id);

        // Albums
        let mut album_query = album::Entity::find()
            .column_as(child::Column::Id.count(), "song_count")
            .column_as(child::Column::Duration.sum(), "duration")
            .column_as(child::Column::PlayCount.sum(), "play_count")
            .column_as(child::Column::LastPlayed.max(), "last_played")
            .join_rev(
                JoinType::LeftJoin,
                child::Entity::belongs_to(album::Entity)
                    .from(child::Column::AlbumId)
                    .to(album::Column::Id)
                    .into(),
            )
            .filter(album::Column::Name.like(&search_query))
            .group_by(album::Column::Id);

        let mut song_query = child::Entity::find()
            .filter(child::Column::IsDir.eq(false))
            .filter(
                child::Column::Title.like(&search_query)
                    .or(child::Column::Album.like(&search_query))
                    .or(child::Column::Artist.like(&search_query))
            );

        if let Some(folder_id) = opts.music_folder_id {
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
                        .and_where(child::Column::MusicFolderId.eq(folder_id))
                        .to_owned(),
                ),
            );

            album_query = album_query.filter(child::Column::MusicFolderId.eq(folder_id));

            song_query = song_query.filter(child::Column::MusicFolderId.eq(folder_id));
        }

        let artists = artist_query
            .limit(opts.artist_count)
            .offset(opts.artist_offset)
            .into_model::<ArtistWithStats>()
            .all(&self.db)
            .await?;
        
        let albums = album_query
            .limit(opts.album_count)
            .offset(opts.album_offset)
            .into_model::<AlbumWithStats>()
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
        let clean_query = query.trim().trim_matches('"');
        let search_query = format!("%{}%", clean_query);
        
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
