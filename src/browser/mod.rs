use sea_orm::{DatabaseConnection};

pub mod types;
pub mod library;
pub mod browsing;
pub mod search;
pub mod playlists;
pub mod bookmarks;
pub mod utils;

pub use crate::browser::types::*;

pub struct Browser {
    pub(crate) db: DatabaseConnection,
}

impl Browser {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
