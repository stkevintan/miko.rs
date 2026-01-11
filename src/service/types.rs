pub use crate::models::queries::ChildWithMetadata;
use crate::models::{child, bookmark};
use serde::Deserialize;
use sea_orm::FromQueryResult;

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlbumListOptions {
    pub r#type: Option<String>,
    pub size: Option<u64>,
    pub offset: Option<u64>,
    pub genre: Option<String>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub music_folder_id: Option<i32>,
}

#[derive(Debug, FromQueryResult)]
pub struct GenreWithStats {
    pub value: String,
    pub song_count: i32,
    pub album_count: i32,
}

#[derive(Debug, FromQueryResult, Clone)]
pub struct ArtistWithStats {
    pub id: String,
    pub name: String,
    pub artist_image_url: Option<String>,
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    pub user_rating: i32,
    pub average_rating: f64,
    pub album_count: i64,
}

#[derive(Debug, FromQueryResult, Clone)]
pub struct PlaylistWithStats {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub owner: String,
    pub public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub song_count: i64,
    pub duration: i64,
}

pub struct PlaylistWithSongs {
    pub playlist: PlaylistWithStats,
    pub entry: Vec<ChildWithMetadata>,
}

pub struct DirectoryWithChildren {
    pub dir: child::Model,
    pub children: Vec<ChildWithMetadata>,
    pub total_count: i64,
}

#[derive(Debug, Clone)]
pub struct BookmarkWithMetadata {
    pub b_username: String,
    pub b_song_id: String,
    pub b_position: i64,
    pub b_comment: Option<String>,
    pub b_created_at: chrono::DateTime<chrono::Utc>,
    pub b_updated_at: chrono::DateTime<chrono::Utc>,
    pub child: ChildWithMetadata,
}

impl FromQueryResult for BookmarkWithMetadata {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
        Ok(Self {
            b_username: res.try_get(pre, "b_username")?,
            b_song_id: res.try_get(pre, "b_song_id")?,
            b_position: res.try_get(pre, "b_position")?,
            b_comment: res.try_get(pre, "b_comment")?,
            b_created_at: res.try_get(pre, "b_created_at")?,
            b_updated_at: res.try_get(pre, "b_updated_at")?,
            child: ChildWithMetadata::from_query_result(res, pre)?,
        })
    }
}

impl From<BookmarkWithMetadata> for (bookmark::Model, ChildWithMetadata) {
    fn from(r: BookmarkWithMetadata) -> Self {
        (
            bookmark::Model {
                username: r.b_username,
                song_id: r.b_song_id,
                position: r.b_position,
                comment: r.b_comment,
                created_at: r.b_created_at,
                updated_at: r.b_updated_at,
            },
            r.child,
        )
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaylistOptions {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    pub song_ids_to_add: Vec<String>,
    pub song_indices_to_remove: Vec<i32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub query: String,
    pub artist_count: u64,
    pub artist_offset: u64,
    pub album_count: u64,
    pub album_offset: u64,
    pub song_count: u64,
    pub song_offset: u64,
    pub music_folder_id: Option<i32>,
}
