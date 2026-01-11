use sea_orm::{DatabaseConnection};

pub mod types;
pub mod library;
pub mod browsing;
pub mod search;
pub mod playlists;
pub mod bookmarks;
pub mod lyrics;
pub mod utils;

pub use crate::service::types::*;

pub struct Service {
    pub(crate) db: DatabaseConnection,
}

impl Service {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
