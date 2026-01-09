use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "genres")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::song_genre::Entity")]
    SongGenre,
    #[sea_orm(has_many = "super::album_genre::Entity")]
    AlbumGenre,
}

impl Related<super::child::Entity> for Entity {
    fn to() -> RelationDef {
        super::song_genre::Relation::Child.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::song_genre::Relation::Genre.def().rev())
    }
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef {
        super::album_genre::Relation::Album.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::album_genre::Relation::Genre.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
