use crate::models::{album, album_artist, album_genre, artist, child, lyrics, song_artist};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, EntityTrait, FromQueryResult, JoinType, QueryFilter, QuerySelect, RelationTrait,
};

#[derive(Debug, FromQueryResult, Clone)]
pub struct LyricsWithMetadata {
    pub title: String,
    pub artist: Option<String>,
    pub content: String,
}

#[derive(sea_orm::FromQueryResult)]
pub struct SongPathInfo {
    pub path: String,
    pub music_folder_id: i32,
}

#[derive(sea_orm::FromQueryResult)]
pub struct FolderPathInfo {
    pub path: String,
}

pub fn song_path_info_query() -> sea_orm::Select<child::Entity> {
    child::Entity::find()
        .filter(child::Column::IsDir.eq(false))
        .filter(child::Column::ContentType.starts_with("audio/"))
        .select_only()
        .column(child::Column::Path)
        .column(child::Column::MusicFolderId)
}

pub fn lyrics_with_metadata_query() -> sea_orm::Select<lyrics::Entity> {
    lyrics::Entity::find()
        .join(JoinType::InnerJoin, lyrics::Relation::Child.def())
        .column_as(child::Column::Title, "title")
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
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name)"), "artist")
        .group_by(lyrics::Column::SongId)
}

pub fn song_with_metadata_query() -> sea_orm::Select<child::Entity> {
    child::Entity::find()
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(a.id || '[:]' || a.name) FROM song_artists sa JOIN artists a ON sa.artist_id = a.id WHERE sa.song_id = children.id)"), "artists")
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(a.id || '[:]' || a.name) FROM album_artists aa JOIN artists a ON aa.artist_id = a.id WHERE aa.album_id = children.album_id)"), "album_artists")
        .column_as(Expr::cust("(SELECT GROUP_CONCAT(genre_name) FROM song_genres WHERE song_id = children.id)"), "genre")
        .column_as(Expr::cust("(SELECT name FROM albums WHERE id = children.album_id)"), "album")
}

pub fn artist_with_stats_query() -> sea_orm::Select<artist::Entity> {
    artist::Entity::find()
        .column_as(Expr::cust("(SELECT COUNT(DISTINCT album_id) FROM (SELECT album_id FROM children JOIN song_artists ON song_artists.song_id = children.id WHERE song_artists.artist_id = artists.id UNION SELECT album_id FROM album_artists WHERE album_artists.artist_id = artists.id))"), "album_count")
}

pub fn album_with_stats_query() -> sea_orm::Select<album::Entity> {
    album::Entity::find()
        .column_as(child::Column::Id.count(), "song_count")
        .column_as(Expr::cust("COALESCE(SUM(duration), 0)"), "duration")
        .column_as(Expr::cust("COALESCE(SUM(play_count), 0)"), "play_count")
        .column_as(child::Column::LastPlayed.max(), "last_played")
        .column_as(
            Expr::cust("GROUP_CONCAT(DISTINCT artists.id || '[:]' || artists.name)"),
            "artists",
        )
        .column_as(
            Expr::cust("GROUP_CONCAT(DISTINCT album_genres.genre_name)"),
            "genre",
        )
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
