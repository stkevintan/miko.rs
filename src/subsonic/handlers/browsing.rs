use actix_web::{web, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use crate::subsonic::models::{SubsonicResponse, MusicFolders, MusicFolder, Indexes, Index, Artist, Directory, Child as SubsonicChild, Genres, Genre, ArtistsID3, IndexID3, ArtistID3, ArtistWithAlbumsID3, AlbumID3, AlbumWithSongsID3, MusicFoldersBody, IndexesBody, DirectoryBody, GenresBody, ArtistsBody, ArtistBody, AlbumBody, SongBody};
use crate::subsonic::handlers::{SubsonicParams, send_response};
use crate::models::{music_folder, child, artist, album, genre};
use std::collections::HashMap;

pub async fn get_music_folders(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    
    let folders = match music_folder::Entity::find().all(db.get_ref()).await {
        Ok(f) => f,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to fetch music folders".into(), version), &params.f),
    };

    let resp = SubsonicResponse::new_ok(MusicFoldersBody {
        music_folders: MusicFolders {
            music_folder: folders.into_iter().map(|f| MusicFolder {
                id: f.id,
                name: f.name,
            }).collect(),
        }
    }, version);
    
    send_response(resp, &params.f)
}

pub async fn get_indexes(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let music_folder_id = query.get("musicFolderId").and_then(|id| id.parse::<i32>().ok());

    let mut db_query = child::Entity::find()
        .filter(child::Column::IsDir.eq(true))
        .filter(child::Column::Parent.eq(""));

    if let Some(folder_id) = music_folder_id {
        db_query = db_query.filter(child::Column::MusicFolderId.eq(folder_id));
    }

    let children = match db_query.all(db.get_ref()).await {
        Ok(c) => c,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query indexes".into(), version), &params.f),
    };

    let mut index_map: HashMap<String, Vec<Artist>> = HashMap::new();
    for child in children {
        if child.title.is_empty() { continue; }
        let first_char = child.title.chars().next().unwrap().to_uppercase().to_string();
        index_map.entry(first_char).or_default().push(Artist {
            id: child.id,
            name: child.title,
            artist_image_url: None,
            starred: None,
            user_rating: None,
            average_rating: None,
        });
    }

    let mut indexes_vec: Vec<Index> = index_map.into_iter().map(|(name, mut artists)| {
        artists.sort_by(|a, b| a.name.cmp(&b.name));
        Index { name, artist: artists }
    }).collect();
    indexes_vec.sort_by(|a, b| a.name.cmp(&b.name));

    let resp = SubsonicResponse::new_ok(IndexesBody {
        indexes: Indexes {
            last_modified: 0, // TODO: Get last scan time
            ignored_articles: "".into(),
            shortcut: vec![],
            index: indexes_vec,
            child: vec![],
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_music_directory(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let id = match query.get("id") {
        Some(id) => id,
        None => return send_response(SubsonicResponse::new_error(10, "ID is required".into(), version), &params.f),
    };

    let dir = match child::Entity::find_by_id(id).filter(child::Column::IsDir.eq(true)).one(db.get_ref()).await {
        Ok(Some(d)) => d,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Directory not found".into(), version), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into(), version), &params.f),
    };

    let children = match child::Entity::find()
        .filter(child::Column::Parent.eq(dir.id.clone()))
        .order_by_desc(child::Column::IsDir)
        .order_by_asc(child::Column::Title)
        .all(db.get_ref()).await {
        Ok(c) => c,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query directory".into(), version), &params.f),
    };

    let resp = SubsonicResponse::new_ok(DirectoryBody {
        directory: Directory {
            id: dir.id,
            parent: if dir.parent.is_empty() { None } else { Some(dir.parent) },
            name: dir.title,
            starred: dir.starred,
            user_rating: Some(dir.user_rating),
            average_rating: Some(dir.average_rating),
            play_count: Some(dir.play_count),
            child: children.into_iter().map(map_child_to_subsonic).collect(),
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_genres(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    
    // Simplified genre query for now
    let genres = match genre::Entity::find().all(db.get_ref()).await {
        Ok(g) => g,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query genres".into(), version), &params.f),
    };

    let resp = SubsonicResponse::new_ok(GenresBody {
        genres: Genres {
            genre: genres.into_iter().map(|g| Genre {
                value: g.name,
                song_count: 0, // TODO
                album_count: 0, // TODO
            }).collect(),
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_artists(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    
    let artists = match artist::Entity::find().all(db.get_ref()).await {
        Ok(a) => a,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query artists".into(), version), &params.f),
    };

    let mut index_map: HashMap<String, Vec<ArtistID3>> = HashMap::new();
    for a in artists {
        if a.name.is_empty() { continue; }
        let first_char = a.name.chars().next().unwrap().to_uppercase().to_string();
        index_map.entry(first_char).or_default().push(ArtistID3 {
            id: a.id,
            name: a.name,
            cover_art: if a.cover_art.is_empty() { None } else { Some(a.cover_art) },
            artist_image_url: if a.artist_image_url.is_empty() { None } else { Some(a.artist_image_url) },
            album_count: 0, // TODO
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
        });
    }

    let mut index_vec: Vec<IndexID3> = index_map.into_iter().map(|(name, mut artists)| {
        artists.sort_by(|a, b| a.name.cmp(&b.name));
        IndexID3 { name, artist: artists }
    }).collect();
    index_vec.sort_by(|a, b| a.name.cmp(&b.name));

    let resp = SubsonicResponse::new_ok(ArtistsBody {
        artists: ArtistsID3 {
            ignored_articles: "".into(),
            index: index_vec,
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_artist(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let id = match query.get("id") {
        Some(id) => id,
        None => return send_response(SubsonicResponse::new_error(10, "ID is required".into(), version), &params.f),
    };

    let artist = match artist::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(a)) => a,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Artist not found".into(), version), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into(), version), &params.f),
    };

    // Fetch albums for this artist
    let albums = match album::Entity::find().filter(album::Column::ArtistId.eq(artist.id.clone())).all(db.get_ref()).await {
        Ok(al) => al,
        Err(_) => vec![],
    };

    let resp = SubsonicResponse::new_ok(ArtistBody {
        artist: ArtistWithAlbumsID3 {
            artist: ArtistID3 {
                id: artist.id,
                name: artist.name,
                cover_art: if artist.cover_art.is_empty() { None } else { Some(artist.cover_art) },
                artist_image_url: if artist.artist_image_url.is_empty() { None } else { Some(artist.artist_image_url) },
                album_count: albums.len() as i32,
                starred: artist.starred,
                user_rating: Some(artist.user_rating),
                average_rating: Some(artist.average_rating),
            },
            album: albums.into_iter().map(|al| AlbumID3 {
                id: al.id,
                name: al.name,
                artist: Some(al.artist),
                artist_id: Some(al.artist_id),
                cover_art: if al.cover_art.is_empty() { None } else { Some(al.cover_art) },
                song_count: 0, // TODO
                duration: 0, // TODO
                play_count: None,
                created: al.created,
                starred: al.starred,
                user_rating: Some(al.user_rating),
                average_rating: Some(al.average_rating),
                year: Some(al.year),
                genre: Some(al.genre),
            }).collect(),
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_album(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let id = match query.get("id") {
        Some(id) => id,
        None => return send_response(SubsonicResponse::new_error(10, "ID is required".into(), version), &params.f),
    };

    let album = match album::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(al)) => al,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Album not found".into(), version), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into(), version), &params.f),
    };

    let songs = match child::Entity::find().filter(child::Column::AlbumId.eq(album.id.clone())).order_by_asc(child::Column::DiscNumber).order_by_asc(child::Column::Track).all(db.get_ref()).await {
        Ok(s) => s,
        Err(_) => vec![],
    };

    let resp = SubsonicResponse::new_ok(AlbumBody {
        album: AlbumWithSongsID3 {
            album: AlbumID3 {
                id: album.id,
                name: album.name,
                artist: Some(album.artist),
                artist_id: Some(album.artist_id),
                cover_art: if album.cover_art.is_empty() { None } else { Some(album.cover_art) },
                song_count: songs.len() as i32,
                duration: songs.iter().map(|s| s.duration).sum(),
                play_count: None,
                created: album.created,
                starred: album.starred,
                user_rating: Some(album.user_rating),
                average_rating: Some(album.average_rating),
                year: Some(album.year),
                genre: Some(album.genre),
            },
            song: songs.into_iter().map(map_child_to_subsonic).collect(),
        }
    }, version);

    send_response(resp, &params.f)
}

pub async fn get_song(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let version = params.v.clone().unwrap_or_else(|| "1.16.1".to_string());
    let id = match query.get("id") {
        Some(id) => id,
        None => return send_response(SubsonicResponse::new_error(10, "ID is required".into(), version), &params.f),
    };

    let song = match child::Entity::find_by_id(id).filter(child::Column::IsDir.eq(false)).one(db.get_ref()).await {
        Ok(Some(s)) => s,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Song not found".into(), version), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into(), version), &params.f),
    };

    let resp = SubsonicResponse::new_ok(SongBody {
        song: map_child_to_subsonic(song)
    }, version);

    send_response(resp, &params.f)
}

fn map_child_to_subsonic(c: child::Model) -> SubsonicChild {
    SubsonicChild {
        id: c.id,
        parent: if c.parent.is_empty() { None } else { Some(c.parent) },
        is_dir: c.is_dir,
        title: c.title,
        album: if c.album.is_empty() { None } else { Some(c.album) },
        artist: if c.artist.is_empty() { None } else { Some(c.artist) },
        track: Some(c.track),
        year: Some(c.year),
        genre: if c.genre.is_empty() { None } else { Some(c.genre) },
        cover_art: if c.cover_art.is_empty() { None } else { Some(c.cover_art) },
        size: Some(c.size),
        content_type: Some(c.content_type),
        suffix: Some(c.suffix),
        transcoded_content_type: if c.transcoded_content_type.is_empty() { None } else { Some(c.transcoded_content_type) },
        transcoded_suffix: if c.transcoded_suffix.is_empty() { None } else { Some(c.transcoded_suffix) },
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
        album_id: if c.album_id.is_empty() { None } else { Some(c.album_id) },
        artist_id: if c.artist_id.is_empty() { None } else { Some(c.artist_id) },
        r#type: Some(c.r#type),
        bookmark_position: None,
    }
}
