use crate::models::{artist, child, genre, song_genre, album, song_artist, album_artist, album_genre, lyrics};
use sea_orm::{FromQueryResult, JoinType, QuerySelect, RelationTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::sea_query::Expr;

#[derive(Debug, FromQueryResult, Clone)]
pub struct ChildWithMetadata {
    pub id: String,
    pub parent: Option<String>,
    pub is_dir: bool,
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub track: i32,
    pub year: i32,
    pub genre: Option<String>,
    pub size: i64,
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    pub transcoded_content_type: Option<String>,
    pub transcoded_suffix: Option<String>,
    pub duration: i32,
    pub bit_rate: i32,
    pub path: String,
    pub is_video: bool,
    pub user_rating: i32,
    pub average_rating: f64,
    pub play_count: i64,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub disc_number: i32,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub r#type: String,
}

#[derive(Debug, FromQueryResult)]
pub struct AlbumWithStats {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    pub user_rating: i32,
    pub average_rating: f64,
    pub year: i32,
    pub genre: Option<String>,
    pub song_count: i64,
    pub duration: i64,
    pub play_count: i64,
}

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
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name, ', ')"), "artist")
        .group_by(lyrics::Column::SongId)
}

pub fn song_with_metadata_query() -> sea_orm::Select<child::Entity> {
    child::Entity::find()
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name, ', ')"), "artist")
        .column_as(Expr::cust("MIN(artists.id)"), "artist_id")
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT genres.name, ', ')"), "genre")
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
        .column_as(Expr::cust("COALESCE(SUM(duration), 0)"), "duration")
        .column_as(Expr::cust("COALESCE(SUM(play_count), 0)"), "play_count")
        .column_as(child::Column::LastPlayed.max(), "last_played")
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.name, ', ')"), "artist")
        .column_as(Expr::cust("MIN(artists.id)"), "artist_id")
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT album_genres.genre_name, ', ')"), "genre")
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
