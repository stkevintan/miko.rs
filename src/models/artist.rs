use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "artists")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    pub starred: Option<DateTimeUtc>,
    #[sea_orm(default_value = 0)]
    pub user_rating: i32,
    #[sea_orm(default_value = 0.0)]
    pub average_rating: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::song_artist::Entity")]
    SongArtist,
}

impl Related<super::child::Entity> for Entity {
    fn to() -> RelationDef {
        super::song_artist::Relation::Child.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::song_artist::Relation::Artist.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
