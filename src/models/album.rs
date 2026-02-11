use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::artist::{parse_artists_field, ArtistIdName};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "albums")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub created: DateTimeUtc,
    pub starred: Option<DateTimeUtc>,
    #[sea_orm(default_value = 0)]
    pub user_rating: i32,
    #[sea_orm(default_value = 0.0)]
    pub average_rating: f64,
    #[sea_orm(default_value = 0)]
    pub year: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::album_genre::Entity")]
    AlbumGenre,
}

impl Related<super::genre::Entity> for Entity {
    fn to() -> RelationDef {
        super::album_genre::Relation::Genre.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::album_genre::Relation::Album.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone)]
pub struct AlbumWithStats {
    pub id: String,
    pub name: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    pub user_rating: i32,
    pub average_rating: f64,
    pub year: i32,
    pub genre: Option<String>,
    pub song_count: i64,
    pub duration: i64,
    pub play_count: i64,
    pub artists: Vec<ArtistIdName>,
}

impl FromQueryResult for AlbumWithStats {
    fn from_query_result(res: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
        let artists = parse_artists_field(res, pre, "artists")?;

        Ok(Self {
            id: res.try_get(pre, "id")?,
            name: res.try_get(pre, "name")?,
            created: res.try_get(pre, "created")?,
            starred: res.try_get(pre, "starred")?,
            user_rating: res.try_get(pre, "user_rating")?,
            average_rating: res.try_get(pre, "average_rating")?,
            year: res.try_get(pre, "year")?,
            genre: res.try_get(pre, "genre")?,
            song_count: res.try_get(pre, "song_count")?,
            duration: res.try_get(pre, "duration")?,
            play_count: res.try_get(pre, "play_count")?,
            artists,
        })
    }
}
