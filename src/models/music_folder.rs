use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "music_folders")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub path: String,
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_music_folder::Entity")]
    UserMusicFolders,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_music_folder::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_music_folder::Relation::MusicFolder.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
