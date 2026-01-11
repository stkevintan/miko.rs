use crate::models::{artist, child, album, song_artist, album_artist, album_genre, lyrics};
use sea_orm::{FromQueryResult, JoinType, QuerySelect, RelationTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::sea_query::Expr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtistIdName {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenreName {
    #[serde(rename = "@name")]
    pub name: String,
}

fn parse_artists_field(res: &sea_orm::QueryResult, pre: &str, col: &str) -> Result<Vec<ArtistIdName>, sea_orm::DbErr> {
    let artists_raw: Option<String> = res.try_get(pre, col)?;
    Ok(artists_raw
        .map(|s| {
            s.split(',')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, "[:]");
                    let id = parts.next()?.to_string();
                    let name = parts.next()?.to_string();
                    Some(ArtistIdName { id, name })
                })
                .collect()
        })
        .unwrap_or_default())
}

#[derive(Debug, Clone)]
pub struct ChildWithMetadata {
    pub id: String,
    pub parent: Option<String>,
    pub is_dir: bool,
    pub title: String,
    pub album: Option<String>,
    pub track: i32,
    pub year: i32,
    pub genre: Option<String>,
    pub genres: Vec<GenreName>,
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
    pub r#type: String,
    pub artists: Vec<ArtistIdName>,
    pub album_artists: Vec<ArtistIdName>,
}

fn parse_genres_field(res: &sea_orm::QueryResult, pre: &str, col: &str) -> Result<Vec<GenreName>, sea_orm::DbErr> {
    let genre_raw: Option<String> = res.try_get(pre, col)?;
    Ok(genre_raw
        .map(|s| s.split(',').map(|s| GenreName { name: s.to_string() }).collect())
        .unwrap_or_default())
}

impl FromQueryResult for ChildWithMetadata {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
        let artists = parse_artists_field(res, pre, "artists")?;
        let album_artists = parse_artists_field(res, pre, "album_artists")?;
        let genres = parse_genres_field(res, pre, "genre")?;

        Ok(Self {
            id: res.try_get(pre, "id")?,
            parent: res.try_get(pre, "parent")?,
            is_dir: res.try_get(pre, "is_dir")?,
            title: res.try_get(pre, "title")?,
            album: res.try_get(pre, "album")?,
            track: res.try_get(pre, "track")?,
            year: res.try_get(pre, "year")?,
            genre: res.try_get(pre, "genre")?,
            genres,
            size: res.try_get(pre, "size")?,
            content_type: res.try_get(pre, "content_type")?,
            suffix: res.try_get(pre, "suffix")?,
            transcoded_content_type: res.try_get(pre, "transcoded_content_type")?,
            transcoded_suffix: res.try_get(pre, "transcoded_suffix")?,
            duration: res.try_get(pre, "duration")?,
            bit_rate: res.try_get(pre, "bit_rate")?,
            path: res.try_get(pre, "path")?,
            is_video: res.try_get(pre, "is_video")?,
            user_rating: res.try_get(pre, "user_rating")?,
            average_rating: res.try_get(pre, "average_rating")?,
            play_count: res.try_get(pre, "play_count")?,
            last_played: res.try_get(pre, "last_played")?,
            disc_number: res.try_get(pre, "disc_number")?,
            created: res.try_get(pre, "created")?,
            starred: res.try_get(pre, "starred")?,
            album_id: res.try_get(pre, "album_id")?,
            r#type: res.try_get(pre, "type")?,
            artists,
            album_artists,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AlbumWithStats {
    pub id: String,
    pub name: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    pub user_rating: i32,
    pub average_rating: f64,
    pub year: i32,
    pub genre: Option<String>,
    pub song_count: i64,
    pub duration: i64,
    pub play_count: i64,
    pub artists: Vec<ArtistIdName>,
}

impl FromQueryResult for AlbumWithStats {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
        let artists = parse_artists_field(res, pre, "artists")?;

        Ok(Self {
            id: res.try_get(pre, "id")?,
            name: res.try_get(pre, "name")?,
            created: res.try_get(pre, "created")?,
            starred: res.try_get(pre, "starred")?,
            user_rating: res.try_get(pre, "user_rating")?,
            average_rating: res.try_get(pre, "average_rating")?,
            year: res.try_get(pre, "year")?,
            genre: res.try_get(pre, "genre")?,
            song_count: res.try_get(pre, "song_count")?,
            duration: res.try_get(pre, "duration")?,
            play_count: res.try_get(pre, "play_count")?,
            artists,
        })
    }
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
        .column_as(Expr::cust("GROUP_CONCAT(DISTINCT artists.id || '[:]' || artists.name)"), "artists")
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
