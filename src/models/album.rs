use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub cover_art: String,
    pub created: DateTimeUtc,
    pub starred: Option<DateTimeUtc>,
    pub user_rating: i32,
    pub average_rating: f64,
    pub year: i32,
    pub genre: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
