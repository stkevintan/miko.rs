use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

pub use super::artist::{ArtistIdName, parse_artists_field};
pub use super::genre::{GenreName, parse_genres_field};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "children")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(index)]
    pub parent: Option<String>,
    pub is_dir: bool,
    #[sea_orm(index)]
    pub title: String,
    #[sea_orm(default_value = 0)]
    pub track: i32,
    #[sea_orm(default_value = 0)]
    pub year: i32,
    #[sea_orm(default_value = 0)]
    pub size: i64,
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    pub transcoded_content_type: Option<String>,
    pub transcoded_suffix: Option<String>,
    #[sea_orm(default_value = 0)]
    pub duration: i32,
    #[sea_orm(default_value = 0)]
    pub bit_rate: i32,
    #[sea_orm(unique)]
    pub path: String,
    #[sea_orm(default_value = false)]
    pub is_video: bool,
    #[sea_orm(default_value = 0)]
    pub user_rating: i32,
    #[sea_orm(default_value = 0.0)]
    pub average_rating: f64,
    #[sea_orm(default_value = 0)]
    pub play_count: i64,
    pub last_played: Option<DateTimeUtc>,
    #[sea_orm(default_value = 0)]
    pub disc_number: i32,
    pub created: Option<DateTimeUtc>,
    pub starred: Option<DateTimeUtc>,
    #[sea_orm(index)]
    pub album_id: Option<String>,
    #[sea_orm(index)]
    pub music_folder_id: i32,
    #[sea_orm(default_value = "music")]
    pub r#type: String,
    #[sea_orm(ignore)]
    pub bookmark_position: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::song_artist::Entity")]
    SongArtist,
    #[sea_orm(has_many = "super::song_genre::Entity")]
    SongGenre,
    #[sea_orm(
        belongs_to = "super::album::Entity",
        from = "Column::AlbumId",
        to = "super::album::Column::Id"
    )]
    Album,
    #[sea_orm(has_many = "super::bookmark::Entity")]
    Bookmarks,
    #[sea_orm(has_many = "super::play_queue_song::Entity")]
    PlayQueueSongs,
}

impl Related<super::bookmark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookmarks.def()
    }
}

impl Related<super::play_queue_song::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayQueueSongs.def()
    }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef {
        super::song_artist::Relation::Artist.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::song_artist::Relation::Child.def().rev())
    }
}

impl Related<super::genre::Entity> for Entity {
    fn to() -> RelationDef {
        super::song_genre::Relation::Genre.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::song_genre::Relation::Child.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

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

impl Entity {
    pub async fn count_songs(db: &DatabaseConnection) -> i64 {
        match Entity::find()
            .filter(Column::IsDir.eq(false))
            .count(db)
            .await {
                Ok(c) => c as i64,
                Err(e) => {
                    log::error!("Failed to count songs: {}", e);
                    0
                }
            }
    }
}
