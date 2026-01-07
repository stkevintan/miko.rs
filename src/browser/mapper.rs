use crate::browser::{
    AlbumWithStats, ArtistWithStats, GenreWithStats, PlaylistWithSongs, PlaylistWithStats,
};
use crate::models::{artist, child};
use crate::subsonic::models::{self, AlbumID3, Artist, ArtistID3, Child};



pub fn map_album_to_id3(a: AlbumWithStats) -> AlbumID3 {
    AlbumID3 {
        id: a.id,
        name: a.name,
        artist: Some(a.artist),
        artist_id: Some(a.artist_id),
        cover_art: (!a.cover_art.is_empty()).then_some(a.cover_art),
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

pub fn map_album_to_child(a: AlbumWithStats) -> Child {
    Child {
        id: a.id,
        parent: None,
        is_dir: true,
        title: a.name,
        album: None,
        artist: Some(a.artist),
        track: None,
        year: Some(a.year),
        genre: Some(a.genre),
        cover_art: (!a.cover_art.is_empty()).then_some(a.cover_art),
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

pub fn map_artist_to_subsonic(a: artist::Model) -> Artist {
    Artist {
        id: a.id,
        name: a.name,
        artist_image_url: (!a.artist_image_url.is_empty()).then_some(a.artist_image_url),
        starred: a.starred,
        user_rating: Some(a.user_rating),
        average_rating: Some(a.average_rating),
    }
}

// pub fn map_artist_to_id3(a: artist::Model) -> ArtistID3 {
//     ArtistID3 {
//         id: a.id,
//         name: a.name,
//         cover_art: (!a.cover_art.is_empty()).then_some(a.cover_art),
//         artist_image_url: (!a.artist_image_url.is_empty()).then_some(a.artist_image_url),
//         album_count: 0,
//         starred: a.starred,
//         user_rating: Some(a.user_rating),
//         average_rating: Some(a.average_rating),
//     }
// }

pub fn map_child_to_subsonic(c: child::Model) -> Child {
    Child {
        id: c.id,
        parent: (!c.parent.is_empty()).then_some(c.parent),
        is_dir: c.is_dir,
        title: c.title,
        album: (!c.album.is_empty()).then_some(c.album),
        artist: (!c.artist.is_empty()).then_some(c.artist),
        track: Some(c.track),
        year: Some(c.year),
        genre: (!c.genre.is_empty()).then_some(c.genre),
        cover_art: (!c.cover_art.is_empty()).then_some(c.cover_art),
        size: Some(c.size),
        content_type: Some(c.content_type),
        suffix: Some(c.suffix),
        transcoded_content_type: (!c.transcoded_content_type.is_empty())
            .then_some(c.transcoded_content_type),
        transcoded_suffix: (!c.transcoded_suffix.is_empty()).then_some(c.transcoded_suffix),
        duration: Some(c.duration),
        bit_rate: Some(c.bit_rate),
        path: Some(c.path),
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
pub fn map_genre_to_subsonic(g: GenreWithStats) -> models::Genre {
    models::Genre {
        value: g.value,
        song_count: g.song_count,
        album_count: g.album_count,
    }
}

pub fn map_artist_with_stats_to_id3(a: ArtistWithStats) -> ArtistID3 {
    ArtistID3 {
        id: a.id,
        name: a.name,
        cover_art: (!a.cover_art.is_empty()).then_some(a.cover_art),
        artist_image_url: (!a.artist_image_url.is_empty()).then_some(a.artist_image_url),
        album_count: a.album_count as i32,
        starred: a.starred,
        user_rating: Some(a.user_rating),
        average_rating: Some(a.average_rating),
    }
}

pub fn map_artist_with_stats_to_subsonic(a: ArtistWithStats) -> Artist {
    Artist {
        id: a.id,
        name: a.name,
        artist_image_url: (!a.artist_image_url.is_empty()).then_some(a.artist_image_url),
        starred: a.starred,
        user_rating: Some(a.user_rating),
        average_rating: Some(a.average_rating),
    }
}

pub fn map_playlist_to_subsonic(p: PlaylistWithStats) -> models::Playlist {
    models::Playlist {
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

pub fn map_playlist_with_songs_to_subsonic(p: PlaylistWithSongs) -> models::Playlist {
    let mut playlist = map_playlist_to_subsonic(p.playlist);
    playlist.entry = p.entry.into_iter().map(map_child_to_subsonic).collect();
    playlist
}
