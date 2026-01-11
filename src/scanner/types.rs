use crate::models::{album, artist, child, genre};

pub struct SongRelations {
    pub song_id: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
    pub lyrics: Option<String>,
}

pub struct AlbumRelations {
    pub album_id: String,
    pub artists: Vec<String>,
    pub genres: Vec<String>,
}

pub enum UpsertMessage {
    Artist(Box<artist::ActiveModel>),
    Album(Box<album::ActiveModel>),
    Genre(Box<genre::ActiveModel>),
    Song(Box<child::ActiveModel>),
    SongRelations(Box<SongRelations>),
    AlbumRelations(Box<AlbumRelations>),
    Seen(String),
    Flush(tokio::sync::oneshot::Sender<()>),
}
