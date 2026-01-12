use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "playlists")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub owner: String,
    pub public: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::playlist_song::Entity")]
    PlaylistSong,
}

impl Related<super::playlist_song::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlaylistSong.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, sea_orm::FromQueryResult, Clone)]
pub struct PlaylistWithStats {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub owner: String,
    pub public: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub song_count: i64,
    pub duration: i64,
}
