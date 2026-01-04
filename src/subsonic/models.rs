use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "subsonic-response")]
pub struct SubsonicResponse<T> {
    #[serde(rename = "@status")]
    pub status: ResponseStatus,
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@serverVersion", skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
    #[serde(rename = "@openSubsonic", skip_serializing_if = "Option::is_none")]
    pub open_subsonic: Option<bool>,
    
    #[serde(flatten)]
    pub body: T,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EmptyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: SubsonicError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseBody {
    pub license: License,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFoldersBody {
    #[serde(rename = "musicFolders")]
    pub music_folders: MusicFolders,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSubsonicExtensionsBody {
    #[serde(rename = "openSubsonicExtensions")]
    pub open_subsonic_extensions: OpenSubsonicExtensions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexesBody {
    pub indexes: Indexes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryBody {
    pub directory: Directory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenresBody {
    pub genres: Genres,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistsBody {
    pub artists: ArtistsID3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistBody {
    pub artist: ArtistWithAlbumsID3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumBody {
    pub album: AlbumWithSongsID3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongBody {
    pub song: Child,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSubsonicExtensions {
    #[serde(rename = "extension")]
    pub extension: Vec<OpenSubsonicExtension>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSubsonicExtension {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@versions")]
    pub versions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicError {
    #[serde(rename = "@code")]
    pub code: i32,
    #[serde(rename = "@message")]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    #[serde(rename = "@valid")]
    pub valid: bool,
    #[serde(rename = "@email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "@licenseExpires", skip_serializing_if = "Option::is_none")]
    pub license_expires: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@trialExpires", skip_serializing_if = "Option::is_none")]
    pub trial_expires: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFolders {
    #[serde(rename = "musicFolder")]
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFolder {
    #[serde(rename = "@id")]
    pub id: i32,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl<T> SubsonicResponse<T> {
    pub fn new_ok(body: T) -> Self {
        Self {
            status: ResponseStatus::Ok,
            version: "1.16.1".to_string(),
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            body,
        }
    }
}

impl SubsonicResponse<ErrorBody> {
    pub fn new_error(code: i32, message: String) -> Self {
        Self {
            status: ResponseStatus::Failed,
            version: "1.16.1".to_string(),
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            body: ErrorBody {
                error: SubsonicError { code, message },
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    #[serde(rename = "@lastModified")]
    pub last_modified: i64,
    #[serde(rename = "@ignoredArticles")]
    pub ignored_articles: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shortcut: Vec<Artist>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub index: Vec<Index>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    #[serde(rename = "@name")]
    pub name: String,
    pub artist: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@parent", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(rename = "@isDir")]
    pub is_dir: bool,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@album", skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@track", skip_serializing_if = "Option::is_none")]
    pub track: Option<i32>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(rename = "@contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(rename = "@suffix", skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(rename = "@transcodedContentType", skip_serializing_if = "Option::is_none")]
    pub transcoded_content_type: Option<String>,
    #[serde(rename = "@transcodedSuffix", skip_serializing_if = "Option::is_none")]
    pub transcoded_suffix: Option<String>,
    #[serde(rename = "@duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(rename = "@bitRate", skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(rename = "@path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "@isVideo", skip_serializing_if = "Option::is_none")]
    pub is_video: Option<bool>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(rename = "@lastPlayed", skip_serializing_if = "Option::is_none")]
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@discNumber", skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<i32>,
    #[serde(rename = "@created", skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@albumId", skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "@bookmarkPosition", skip_serializing_if = "Option::is_none")]
    pub bookmark_position: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genres {
    pub genre: Vec<Genre>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    #[serde(rename = "$value")]
    pub value: String,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@albumCount")]
    pub album_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistsID3 {
    #[serde(rename = "@ignoredArticles")]
    pub ignored_articles: String,
    pub index: Vec<IndexID3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexID3 {
    #[serde(rename = "@name")]
    pub name: String,
    pub artist: Vec<ArtistID3>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistID3 {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@artistImageUrl", skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(rename = "@albumCount")]
    pub album_count: i32,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistWithAlbumsID3 {
    #[serde(flatten)]
    pub artist: ArtistID3,
    pub album: Vec<AlbumID3>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumID3 {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@artistId", skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(rename = "@coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(rename = "@playCount", skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(rename = "@created")]
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "@starred", skip_serializing_if = "Option::is_none")]
    pub starred: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@userRating", skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "@averageRating", skip_serializing_if = "Option::is_none")]
    pub average_rating: Option<f64>,
    #[serde(rename = "@year", skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(rename = "@genre", skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumWithSongsID3 {
    #[serde(flatten)]
    pub album: AlbumID3,
    pub song: Vec<Child>,
}
