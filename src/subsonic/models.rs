use crate::browser::{
    AlbumWithStats, ArtistWithStats, GenreWithStats, PlaylistWithSongs, PlaylistWithStats,
};
use crate::models::{artist, child, user};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "subsonic-response")]
pub struct SubsonicResponse {
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

    #[serde(rename = "$value")]
    pub body: SubsonicResponseBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubsonicResponseBody {
    #[serde(rename = "error")]
    Error(SubsonicError),
    #[serde(rename = "license")]
    License(License),
    #[serde(rename = "musicFolders")]
    MusicFolders(MusicFolders),
    #[serde(rename = "openSubsonicExtensions")]
    OpenSubsonicExtensions(OpenSubsonicExtensions),
    #[serde(rename = "indexes")]
    Indexes(Indexes),
    #[serde(rename = "directory")]
    Directory(Directory),
    #[serde(rename = "genres")]
    Genres(Genres),
    #[serde(rename = "artists")]
    Artists(ArtistsID3),
    #[serde(rename = "artist")]
    Artist(ArtistWithAlbumsID3),
    #[serde(rename = "album")]
    Album(AlbumWithSongsID3),
    #[serde(rename = "song")]
    Song(Child),
    #[serde(rename = "scanStatus")]
    ScanStatus(ScanStatus),
    #[serde(rename = "albumList")]
    AlbumList(AlbumList),
    #[serde(rename = "albumList2")]
    AlbumList2(AlbumList2),
    #[serde(rename = "randomSongs")]
    RandomSongs(RandomSongs),
    #[serde(rename = "songsByGenre")]
    SongsByGenre(SongsByGenre),
    #[serde(rename = "nowPlaying")]
    NowPlaying(NowPlaying),
    #[serde(rename = "starred")]
    Starred(Starred),
    #[serde(rename = "starred2")]
    Starred2(Starred2),
    #[serde(rename = "searchResult")]
    SearchResult(SearchResult),
    #[serde(rename = "searchResult2")]
    SearchResult2(SearchResult2),
    #[serde(rename = "searchResult3")]
    SearchResult3(SearchResult3),
    #[serde(rename = "playlists")]
    Playlists(Playlists),
    #[serde(rename = "playlist")]
    Playlist(Playlist),
    #[serde(rename = "artistInfo")]
    ArtistInfo(ArtistInfo),
    #[serde(rename = "artistInfo2")]
    ArtistInfo2(ArtistInfo2),
    #[serde(rename = "albumInfo")]
    AlbumInfo(AlbumInfo),
    #[serde(rename = "similarSongs")]
    SimilarSongs(SimilarSongs),
    #[serde(rename = "similarSongs2")]
    SimilarSongs2(SimilarSongs2),
    #[serde(rename = "topSongs")]
    TopSongs(TopSongs),
    #[serde(rename = "lyrics")]
    Lyrics(Lyrics),
    #[serde(rename = "lyricsList")]
    LyricsList(LyricsList),
    #[serde(rename = "user")]
    User(User),
    #[serde(rename = "users")]
    Users(Users),
    #[serde(other)]
    None,
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

impl SubsonicResponse {
    pub fn new_ok(body: SubsonicResponseBody) -> Self {
        Self {
            status: ResponseStatus::Ok,
            version: "1.16.1".to_string(),
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            body,
        }
    }

    pub fn new_error(code: i32, message: String) -> Self {
        Self {
            status: ResponseStatus::Failed,
            version: "1.16.1".to_string(),
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            body: SubsonicResponseBody::Error(SubsonicError { code, message }),
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

impl From<artist::Model> for Artist {
    fn from(a: artist::Model) -> Self {
        Self {
            id: a.id,
            name: a.name,
            artist_image_url: a.artist_image_url.filter(|s| !s.is_empty()),
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
        }
    }
}

impl From<ArtistWithStats> for Artist {
    fn from(a: ArtistWithStats) -> Self {
        Self {
            id: a.id,
            name: a.name,
            artist_image_url: a.artist_image_url,
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
        }
    }
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
    #[serde(rename = "@totalCount", skip_serializing_if = "Option::is_none")]
    pub total_count: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Child>,
    #[serde(rename = "parent", skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<Child>,
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
    #[serde(
        rename = "@transcodedContentType",
        skip_serializing_if = "Option::is_none"
    )]
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

impl Child {
    pub fn from_album_stats(a: AlbumWithStats) -> Self {
        Self {
            id: a.id,
            parent: None,
            is_dir: true,
            title: a.name,
            album: None,
            artist: Some(a.artist),
            track: None,
            year: Some(a.year),
            genre: Some(a.genre),
            cover_art: a.cover_art,
            size: None,
            content_type: None,
            suffix: None,
            transcoded_content_type: None,
            transcoded_suffix: None,
            duration: Some(a.duration as i32),
            bit_rate: None,
            path: None,
            is_video: Some(false),
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
            play_count: Some(a.play_count),
            last_played: None,
            disc_number: None,
            created: Some(a.created),
            starred: a.starred,
            album_id: None,
            artist_id: Some(a.artist_id),
            r#type: None,
            bookmark_position: None,
        }
    }
}

impl From<child::Model> for Child {
    fn from(c: child::Model) -> Self {
        Self {
            id: c.id,
            parent: (!c.parent.is_empty()).then_some(c.parent),
            is_dir: c.is_dir,
            title: c.title,
            album: (!c.album.is_empty()).then_some(c.album),
            artist: (!c.artist.is_empty()).then_some(c.artist),
            track: Some(c.track),
            year: Some(c.year),
            genre: (!c.genre.is_empty()).then_some(c.genre),
            cover_art: c.cover_art,
            size: Some(c.size),
            content_type: (!c.content_type.is_empty()).then_some(c.content_type),
            suffix: (!c.suffix.is_empty()).then_some(c.suffix),
            transcoded_content_type: (!c.transcoded_content_type.is_empty())
                .then_some(c.transcoded_content_type),
            transcoded_suffix: (!c.transcoded_suffix.is_empty()).then_some(c.transcoded_suffix),
            duration: Some(c.duration),
            bit_rate: Some(c.bit_rate),
            path: (!c.path.is_empty()).then_some(c.path),
            is_video: Some(c.is_video),
            user_rating: Some(c.user_rating),
            average_rating: Some(c.average_rating),
            play_count: Some(c.play_count),
            last_played: c.last_played,
            disc_number: Some(c.disc_number),
            created: c.created,
            starred: c.starred,
            album_id: (!c.album_id.is_empty()).then_some(c.album_id),
            artist_id: (!c.artist_id.is_empty()).then_some(c.artist_id),
            r#type: Some(c.r#type),
            bookmark_position: None,
        }
    }
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

impl From<GenreWithStats> for Genre {
    fn from(g: GenreWithStats) -> Self {
        Self {
            value: g.value,
            song_count: g.song_count,
            album_count: g.album_count,
        }
    }
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

impl From<ArtistWithStats> for ArtistID3 {
    fn from(a: ArtistWithStats) -> Self {
        Self {
            id: a.id,
            name: a.name,
            cover_art: a.cover_art,
            artist_image_url: a.artist_image_url,
            album_count: a.album_count as i32,
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
        }
    }
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

impl From<AlbumWithStats> for AlbumID3 {
    fn from(a: AlbumWithStats) -> Self {
        Self {
            id: a.id,
            name: a.name,
            artist: Some(a.artist),
            artist_id: Some(a.artist_id),
            cover_art: a.cover_art,
            song_count: a.song_count as i32,
            duration: a.duration as i32,
            play_count: Some(a.play_count),
            created: a.created,
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
            year: Some(a.year),
            genre: Some(a.genre),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumWithSongsID3 {
    #[serde(flatten)]
    pub album: AlbumID3,
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumList {
    pub album: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumList2 {
    pub album: Vec<AlbumID3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RandomSongs {
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongsByGenre {
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NowPlaying {
    #[serde(rename = "entry")]
    pub entry: Vec<NowPlayingEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingEntry {
    #[serde(flatten)]
    pub child: Child,
    #[serde(rename = "@username")]
    pub username: String,
    #[serde(rename = "@minutesAgo")]
    pub minutes_ago: i32,
    #[serde(rename = "@playerId")]
    pub player_id: i32,
    #[serde(rename = "@playerName", skip_serializing_if = "Option::is_none")]
    pub player_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Starred {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<Artist>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Starred2 {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistID3>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStatus {
    #[serde(rename = "@scanning")]
    pub scanning: bool,
    #[serde(rename = "@count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(rename = "@offset")]
    pub offset: u64,
    #[serde(rename = "@totalHits")]
    pub total_hits: u64,
    #[serde(rename = "match")]
    pub match_vec: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult2 {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<Artist>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult3 {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistID3>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlists {
    #[serde(rename = "playlist")]
    pub playlist: Vec<Playlist>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@comment", skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "@owner")]
    pub owner: String,
    #[serde(rename = "@public")]
    pub public: bool,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(rename = "@created")]
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "@changed")]
    pub changed: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "entry", skip_serializing_if = "Vec::is_empty", default)]
    pub entry: Vec<Child>,
}

impl From<PlaylistWithStats> for Playlist {
    fn from(p: PlaylistWithStats) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            comment: (!p.comment.is_empty()).then_some(p.comment),
            owner: p.owner,
            public: p.public,
            song_count: p.song_count as i32,
            duration: p.duration as i32,
            created: p.created_at,
            changed: p.updated_at,
            entry: vec![],
        }
    }
}

impl From<PlaylistWithSongs> for Playlist {
    fn from(p: PlaylistWithSongs) -> Self {
        let mut playlist = Playlist::from(p.playlist);
        playlist.entry = p.entry.into_iter().map(Child::from).collect();
        playlist
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfoBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo {
    #[serde(flatten)]
    pub base: ArtistInfoBase,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub similar_artist: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo2 {
    #[serde(flatten)]
    pub base: ArtistInfoBase,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub similar_artist: Vec<ArtistID3>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SimilarSongs {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SimilarSongs2 {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopSongs {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub song: Vec<Child>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lyrics {
    #[serde(rename = "@artist", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(rename = "@title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LyricsList {
    #[serde(rename = "structuredLyrics")]
    pub structured_lyrics: Vec<StructuredLyrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredLyrics {
    #[serde(rename = "@synced")]
    pub synced: bool,
    #[serde(rename = "@lang", skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "@displayArtist", skip_serializing_if = "Option::is_none")]
    pub display_artist: Option<String>,
    #[serde(rename = "@displayTitle", skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    #[serde(rename = "line")]
    pub lines: Vec<LyricsLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LyricsLine {
    #[serde(rename = "@start", skip_serializing_if = "Option::is_none")]
    pub start: Option<i32>,
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Users {
    #[serde(rename = "user")]
    pub user: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "@username")]
    pub username: String,
    #[serde(rename = "@email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "@scrobblingEnabled")]
    pub scrobbling_enabled: bool,
    #[serde(rename = "@maxBitRate", skip_serializing_if = "Option::is_none")]
    pub max_bit_rate: Option<i32>,
    #[serde(rename = "@adminRole")]
    pub admin_role: bool,
    #[serde(rename = "@settingsRole")]
    pub settings_role: bool,
    #[serde(rename = "@downloadRole")]
    pub download_role: bool,
    #[serde(rename = "@uploadRole")]
    pub upload_role: bool,
    #[serde(rename = "@playlistRole")]
    pub playlist_role: bool,
    #[serde(rename = "@coverArtRole")]
    pub cover_art_role: bool,
    #[serde(rename = "@commentRole")]
    pub comment_role: bool,
    #[serde(rename = "@podcastRole")]
    pub podcast_role: bool,
    #[serde(rename = "@streamRole")]
    pub stream_role: bool,
    #[serde(rename = "@jukeboxRole")]
    pub jukebox_role: bool,
    #[serde(rename = "@shareRole")]
    pub share_role: bool,
    #[serde(rename = "@videoConversionRole")]
    pub video_conversion_role: bool,
    #[serde(rename = "folder", skip_serializing_if = "Vec::is_empty")]
    pub folder: Vec<i32>,
}

impl User {
    pub fn from_db(u: user::Model, folders: Vec<i32>) -> Self {
        Self {
            username: u.username,
            email: u.email,
            scrobbling_enabled: u.scrobbling_enabled,
            max_bit_rate: u.max_bit_rate,
            admin_role: u.admin_role,
            settings_role: u.settings_role,
            download_role: u.download_role,
            upload_role: u.upload_role,
            playlist_role: u.playlist_role,
            cover_art_role: u.cover_art_role,
            comment_role: u.comment_role,
            podcast_role: u.podcast_role,
            stream_role: u.stream_role,
            jukebox_role: u.jukebox_role,
            share_role: u.share_role,
            video_conversion_role: u.video_conversion_role,
            folder: folders,
        }
    }
}
