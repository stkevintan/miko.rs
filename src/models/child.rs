use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "children")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub parent: String,
    pub is_dir: bool,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub track: i32,
    pub year: i32,
    pub genre: String,
    pub cover_art: String,
    pub size: i64,
    pub content_type: String,
    pub suffix: String,
    pub transcoded_content_type: String,
    pub transcoded_suffix: String,
    pub duration: i32,
    pub bit_rate: i32,
    #[sea_orm(unique)]
    pub path: String,
    pub is_video: bool,
    pub user_rating: i32,
    pub average_rating: f64,
    pub play_count: i64,
    pub last_played: Option<DateTimeUtc>,
    pub disc_number: i32,
    pub created: Option<DateTimeUtc>,
    pub starred: Option<DateTimeUtc>,
    pub album_id: String,
    pub artist_id: String,
    pub music_folder_id: i32,
    pub r#type: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
