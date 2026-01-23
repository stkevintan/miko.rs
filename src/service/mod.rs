use sea_orm::{DatabaseConnection};

pub mod library;
pub mod browsing;
pub mod search;
pub mod playlists;
pub mod bookmarks;
pub mod scrape;
pub mod tag;
pub mod musicbrainz;
pub mod utils;

pub struct Service {
    pub(crate) db: DatabaseConnection,
}

impl Service {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
