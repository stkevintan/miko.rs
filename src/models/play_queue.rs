use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "play_queue")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    pub current: Option<String>,
    pub position: i64,
    pub changed: DateTimeUtc,
    pub changed_by: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Username",
        to = "super::user::Column::Username"
    )]
    User,
    #[sea_orm(has_many = "super::play_queue_song::Entity")]
    PlayQueueSongs,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::play_queue_song::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayQueueSongs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
