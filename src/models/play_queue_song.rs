use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "play_queue_song")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub song_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::play_queue::Entity",
        from = "Column::Username",
        to = "super::play_queue::Column::Username"
    )]
    PlayQueue,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Username",
        to = "super::user::Column::Username"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::child::Entity",
        from = "Column::SongId",
        to = "super::child::Column::Id"
    )]
    Child,
}

impl Related<super::play_queue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayQueue.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::child::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Child.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
