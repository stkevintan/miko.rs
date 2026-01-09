use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

static LYRICS_RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^\[(\d+):(\d+)\.(\d+)\](.*)$").unwrap());

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lyrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub song_id: String,
    pub content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::child::Entity",
        from = "Column::SongId",
        to = "super::child::Column::Id"
    )]
    Child,
}

impl Related<super::child::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Child.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn parse(&self) -> (bool, Vec<(Option<i32>, String)>) {
        let mut lines = Vec::new();
        let mut synced = true;

        for row in self.content.lines() {
            let row = row.trim();
            if row.is_empty() {
                continue;
            }

            let parsed_line = (|| {
                let caps = LYRICS_RE.captures(row)?;
                let min: i32 = caps[1].parse().ok()?;
                let sec: i32 = caps[2].parse().ok()?;
                let ms_str = &caps[3];
                let mut ms: i32 = ms_str.parse().ok()?;
                if ms_str.len() == 2 {
                    ms *= 10;
                }
                let text = caps[4].trim().to_string();
                let start_time = (min * 60 + sec) * 1000 + ms;
                Some((Some(start_time), text))
            })();

            match parsed_line {
                Some(line) => lines.push(line),
                None => {
                    synced = false;
                    lines.push((None, row.to_string()));
                }
            }
        }

        (synced, lines)
    }
}
