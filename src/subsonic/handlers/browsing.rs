use actix_web::{web, Responder};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use crate::subsonic::models::{SubsonicResponse, MusicFolders, MusicFolder, Indexes, Index, Artist, Directory, Child as SubsonicChild, Genres, Genre, ArtistsID3, IndexID3, ArtistID3, ArtistWithAlbumsID3, AlbumID3, AlbumWithSongsID3, MusicFoldersBody, IndexesBody, DirectoryBody, GenresBody, ArtistsBody, ArtistBody, AlbumBody, SongBody};
use crate::subsonic::handlers::{SubsonicParams, send_response};
use crate::models::{music_folder, child, artist, album, genre};
use std::collections::{HashMap, BTreeMap};

pub async fn get_music_folders(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    let folders = match music_folder::Entity::find().all(db.get_ref()).await {
        Ok(f) => f,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to fetch music folders".into()), &params.f),
    };

    let resp = SubsonicResponse::new_ok(MusicFoldersBody {
        music_folders: MusicFolders {
            music_folder: folders.into_iter().map(|f| MusicFolder {
                id: f.id,
                name: f.name,
            }).collect(),
        }
    });
    
    send_response(resp, &params.f)
}

pub async fn get_indexes(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let music_folder_id = query.get("musicFolderId").and_then(|id| id.parse::<i32>().ok());

    let mut db_query = child::Entity::find()
        .filter(child::Column::IsDir.eq(true))
        .filter(child::Column::Parent.eq(""));

    if let Some(folder_id) = music_folder_id {
        db_query = db_query.filter(child::Column::MusicFolderId.eq(folder_id));
    }

    let children = match db_query.all(db.get_ref()).await {
        Ok(c) => c,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query indexes".into()), &params.f),
    };

    let mut index_map: BTreeMap<String, Vec<Artist>> = BTreeMap::new();
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

    let indexes_vec: Vec<Index> = index_map.into_iter().map(|(name, mut artists)| {
        artists.sort_by(|a, b| a.name.cmp(&b.name));
        Index { name, artist: artists }
    }).collect();

    let resp = SubsonicResponse::new_ok(IndexesBody {
        indexes: Indexes {
            last_modified: 0, // TODO: Get last scan time
            ignored_articles: "".into(),
            shortcut: vec![],
            index: indexes_vec,
            child: vec![],
        }
    });

    send_response(resp, &params.f)
}

pub async fn get_music_directory(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = get_id_or_error!(query, params);

    let dir = match child::Entity::find_by_id(id).filter(child::Column::IsDir.eq(true)).one(db.get_ref()).await {
        Ok(Some(d)) => d,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Directory not found".into()), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into()), &params.f),
    };

    let children = match child::Entity::find()
        .filter(child::Column::Parent.eq(dir.id.clone()))
        .order_by_desc(child::Column::IsDir)
        .order_by_asc(child::Column::Title)
        .all(db.get_ref()).await {
        Ok(c) => c,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query directory".into()), &params.f),
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
    });

    send_response(resp, &params.f)
}

pub async fn get_genres(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    // Simplified genre query for now
    let genres = match genre::Entity::find().all(db.get_ref()).await {
        Ok(g) => g,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query genres".into()), &params.f),
    };

    let resp = SubsonicResponse::new_ok(GenresBody {
        genres: Genres {
            genre: genres.into_iter().map(|g| Genre {
                value: g.name,
                song_count: 0, // TODO
                album_count: 0, // TODO
            }).collect(),
        }
    });

    send_response(resp, &params.f)
}

pub async fn get_artists(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
) -> impl Responder {
    let artists = match artist::Entity::find().all(db.get_ref()).await {
        Ok(a) => a,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to query artists".into()), &params.f),
    };

    let mut index_map: BTreeMap<String, Vec<ArtistID3>> = BTreeMap::new();
    for a in artists {
        if a.name.is_empty() { continue; }
        let first_char = a.name.chars().next().unwrap().to_uppercase().to_string();
        index_map.entry(first_char).or_default().push(ArtistID3 {
            id: a.id,
            name: a.name,
            cover_art: (!a.cover_art.is_empty()).then_some(a.cover_art),
            artist_image_url: (!a.artist_image_url.is_empty()).then_some(a.artist_image_url),
            album_count: 0, // TODO
            starred: a.starred,
            user_rating: Some(a.user_rating),
            average_rating: Some(a.average_rating),
        });
    }

    let index_vec: Vec<IndexID3> = index_map.into_iter().map(|(name, mut artists)| {
        artists.sort_by(|a, b| a.name.cmp(&b.name));
        IndexID3 { name, artist: artists }
    }).collect();

    let resp = SubsonicResponse::new_ok(ArtistsBody {
        artists: ArtistsID3 {
            ignored_articles: "".into(),
            index: index_vec,
        }
    });

    send_response(resp, &params.f)
}

pub async fn get_artist(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = get_id_or_error!(query, params);

    let artist = match artist::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(a)) => a,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Artist not found".into()), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into()), &params.f),
    };

    // Fetch albums for this artist
    let albums = match album::Entity::find().filter(album::Column::ArtistId.eq(artist.id.clone())).all(db.get_ref()).await {
        Ok(al) => al,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to fetch albums".into()), &params.f),
    };

    let resp = SubsonicResponse::new_ok(ArtistBody {
        artist: ArtistWithAlbumsID3 {
            artist: ArtistID3 {
                id: artist.id,
                name: artist.name,
                cover_art: (!artist.cover_art.is_empty()).then_some(artist.cover_art),
                artist_image_url: (!artist.artist_image_url.is_empty()).then_some(artist.artist_image_url),
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
                cover_art: (!al.cover_art.is_empty()).then_some(al.cover_art),
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
    });

    send_response(resp, &params.f)
}

pub async fn get_album(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = get_id_or_error!(query, params);

    let album = match album::Entity::find_by_id(id).one(db.get_ref()).await {
        Ok(Some(al)) => al,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Album not found".into()), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into()), &params.f),
    };

    let songs = match child::Entity::find().filter(child::Column::AlbumId.eq(album.id.clone())).order_by_asc(child::Column::DiscNumber).order_by_asc(child::Column::Track).all(db.get_ref()).await {
        Ok(s) => s,
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Failed to fetch songs".into()), &params.f),
    };

    let resp = SubsonicResponse::new_ok(AlbumBody {
        album: AlbumWithSongsID3 {
            album: AlbumID3 {
                id: album.id,
                name: album.name,
                artist: Some(album.artist),
                artist_id: Some(album.artist_id),
                cover_art: (!album.cover_art.is_empty()).then_some(album.cover_art),
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
    });

    send_response(resp, &params.f)
}

pub async fn get_song(
    db: web::Data<DatabaseConnection>,
    params: web::Query<SubsonicParams>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = get_id_or_error!(query, params);

    let song = match child::Entity::find_by_id(id).filter(child::Column::IsDir.eq(false)).one(db.get_ref()).await {
        Ok(Some(s)) => s,
        Ok(None) => return send_response(SubsonicResponse::new_error(70, "Song not found".into()), &params.f),
        Err(_) => return send_response(SubsonicResponse::new_error(0, "Database error".into()), &params.f),
    };

    let resp = SubsonicResponse::new_ok(SongBody {
        song: map_child_to_subsonic(song)
    });

    send_response(resp, &params.f)
}

fn map_child_to_subsonic(c: child::Model) -> SubsonicChild {
    SubsonicChild {
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
        transcoded_content_type: (!c.transcoded_content_type.is_empty()).then_some(c.transcoded_content_type),
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
