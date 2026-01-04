use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_music_folders")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub music_folder_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Username",
        to = "super::user::Column::Username"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::music_folder::Entity",
        from = "Column::MusicFolderId",
        to = "super::music_folder::Column::Id"
    )]
    MusicFolder,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::music_folder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MusicFolder.def()
    }
}
