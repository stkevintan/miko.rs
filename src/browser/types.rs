use crate::models::{child};
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
