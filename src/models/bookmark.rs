use sea_orm::entity::prelude::*;
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
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
