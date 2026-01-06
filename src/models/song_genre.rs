use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "song_genres")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub song_id: String,
    #[sea_orm(primary_key)]
    pub genre_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::child::Entity",
        from = "Column::SongId",
        to = "super::child::Column::Id"
    )]
    Child,
    #[sea_orm(
        belongs_to = "super::genre::Entity",
        from = "Column::GenreName",
        to = "super::genre::Column::Name"
    )]
    Genre,
}

impl Related<super::child::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Child.def()
    }
}

impl Related<super::genre::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Genre.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
