use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub created: DateTimeUtc,
    pub starred: Option<DateTimeUtc>,
    #[sea_orm(default_value = 0)]
    pub user_rating: i32,
    #[sea_orm(default_value = 0.0)]
    pub average_rating: f64,
    #[sea_orm(default_value = 0)]
    pub year: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::album_genre::Entity")]
    AlbumGenre,
}

impl Related<super::genre::Entity> for Entity {
    fn to() -> RelationDef {
        super::album_genre::Relation::Genre.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::album_genre::Relation::Album.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
