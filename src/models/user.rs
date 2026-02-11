use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,

    // Subsonic Settings
    #[sea_orm(default_value = true)]
    pub scrobbling_enabled: bool,
    pub max_bit_rate: Option<i32>,
    pub settings_role: bool,
    #[sea_orm(default_value = true)]
    pub download_role: bool,
    pub upload_role: bool,
    pub admin_role: bool,
    #[sea_orm(default_value = true)]
    pub playlist_role: bool,
    #[sea_orm(default_value = true)]
    pub cover_art_role: bool,
    #[sea_orm(default_value = true)]
    pub comment_role: bool,
    pub podcast_role: bool,
    #[sea_orm(default_value = true)]
    pub stream_role: bool,
    pub jukebox_role: bool,
    #[sea_orm(default_value = true)]
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub avatar_last_changed: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_music_folder::Entity")]
    UserMusicFolders,
    #[sea_orm(has_many = "super::bookmark::Entity")]
    Bookmarks,
    #[sea_orm(has_one = "super::play_queue::Entity")]
    PlayQueue,
    #[sea_orm(has_many = "super::play_queue_song::Entity")]
    PlayQueueSongs,
}

impl Related<super::bookmark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookmarks.def()
    }
}

impl Related<super::play_queue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayQueue.def()
    }
}

impl Related<super::play_queue_song::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlayQueueSongs.def()
    }
}

impl Related<super::music_folder::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_music_folder::Relation::MusicFolder.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_music_folder::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
