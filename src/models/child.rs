use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    #[sea_orm(index)]
    pub album: Option<String>,
    #[sea_orm(index)]
    pub artist: Option<String>,
    #[sea_orm(default_value = 0)]
    pub track: i32,
    #[sea_orm(default_value = 0)]
    pub year: i32,
    #[sea_orm(index)]
    pub genre: Option<String>,
    pub lyrics: Option<String>,
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
    pub artist_id: Option<String>,
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
