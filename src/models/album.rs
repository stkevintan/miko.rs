use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(default_value = "")]
    pub artist: String,
    #[sea_orm(default_value = "")]
    pub artist_id: String,
    pub cover_art: Option<String>,
    pub created: DateTimeUtc,
    pub starred: Option<DateTimeUtc>,
    #[sea_orm(default_value = 0)]
    pub user_rating: i32,
    #[sea_orm(default_value = 0.0)]
    pub average_rating: f64,
    #[sea_orm(default_value = 0)]
    pub year: i32,
    #[sea_orm(default_value = "")]
    pub genre: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
