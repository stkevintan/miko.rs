use sea_orm::DatabaseConnection;

pub mod bookmarks;
pub mod browsing;
pub mod library;
pub mod musicbrainz;
pub mod playlists;
pub mod scrape;
pub mod search;
pub mod tag;
pub mod utils;

pub struct Service {
    pub(crate) db: DatabaseConnection,
}

impl Service {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
