use super::child::ChildWithMetadata;
use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bookmark")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub song_id: String,
    pub position: i64,
    pub comment: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
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
        belongs_to = "super::child::Entity",
        from = "Column::SongId",
        to = "super::child::Column::Id"
    )]
    Child,
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

#[derive(Debug, Clone)]
pub struct BookmarkWithMetadata {
    pub b_username: String,
    pub b_song_id: String,
    pub b_position: i64,
    pub b_comment: Option<String>,
    pub b_created_at: DateTimeUtc,
    pub b_updated_at: DateTimeUtc,
    pub child: ChildWithMetadata,
}

impl FromQueryResult for BookmarkWithMetadata {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
        Ok(Self {
            b_username: res.try_get(pre, "b_username")?,
            b_song_id: res.try_get(pre, "b_song_id")?,
            b_position: res.try_get(pre, "b_position")?,
            b_comment: res.try_get(pre, "b_comment")?,
            b_created_at: res.try_get(pre, "b_created_at")?,
            b_updated_at: res.try_get(pre, "b_updated_at")?,
            child: ChildWithMetadata::from_query_result(res, pre)?,
        })
    }
}

impl From<BookmarkWithMetadata> for (Model, ChildWithMetadata) {
    fn from(r: BookmarkWithMetadata) -> Self {
        (
            Model {
                username: r.b_username,
                song_id: r.b_song_id,
                position: r.b_position,
                comment: r.b_comment,
                created_at: r.b_created_at,
                updated_at: r.b_updated_at,
            },
            r.child,
        )
    }
}
