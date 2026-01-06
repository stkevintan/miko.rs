use crate::config::Config;
use crate::models::{album, artist, child, genre, music_folder};
use crate::scanner::Scanner;
use crate::subsonic::{
    common::{SubsonicParams, send_response},
    models::{
        AlbumID3, AlbumWithSongsID3, Artist, ArtistID3, ArtistWithAlbumsID3, ArtistsID3,
        Child as SubsonicChild, Directory, Genre, Genres, Index, IndexID3, Indexes, MusicFolder,
        MusicFolders, SubsonicResponse, SubsonicResponseBody,
    },
};
use poem::{
    handler,
    web::{Data, Query},
    IntoResponse,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[handler]
pub async fn get_music_folders(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let folders = match music_folder::Entity::find().all(*db).await {
        Ok(f) => f,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch music folders".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::MusicFolders(MusicFolders {
        music_folder: folders
            .into_iter()
            .map(|f| MusicFolder {
                id: f.id,
                name: f.name,
            })
            .collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_indexes(
    db: Data<&DatabaseConnection>,
    config: Data<&Arc<Config>>,
    scanner: Data<&Arc<Scanner>>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let music_folder_id = query
        .get("musicFolderId")
        .and_then(|id| id.parse::<i32>().ok());

    let mut db_query = child::Entity::find()
        .filter(child::Column::IsDir.eq(true))
        .filter(child::Column::Parent.eq(""));

    if let Some(folder_id) = music_folder_id {
        db_query = db_query.filter(child::Column::MusicFolderId.eq(folder_id));
    }

    let children = match db_query.all(*db).await {
        Ok(c) => c,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to query indexes".into()),
                &params.f,
            )
        }
    };

    let ignored_articles: Vec<String> = config
        .subsonic
        .ignored_articles
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();

    let mut index_map: BTreeMap<String, Vec<Artist>> = BTreeMap::new();
    let root_children: Vec<SubsonicChild> = Vec::new(); // Subsonic often returns non-grouped folders here

    for child in children {
        if child.title.is_empty() {
            continue;
        }

        let mut display_title = child.title.clone();
        let mut sort_title = display_title.to_lowercase();

        for article in &ignored_articles {
            let article_with_space = format!("{} ", article);
            if sort_title.starts_with(&article_with_space) {
                sort_title = sort_title[article_with_space.len()..].to_string();
                break;
            }
        }

        let first_char = sort_title
            .chars()
            .next()
            .unwrap_or('#')
            .to_uppercase()
            .to_string();

        let index_key = if first_char.chars().next().unwrap().is_alphabetic() {
            first_char
        } else {
            "#".to_string()
        };

        index_map.entry(index_key).or_default().push(Artist {
            id: child.id,
            name: display_title,
            artist_image_url: None,
            starred: None,
            user_rating: None,
            average_rating: None,
        });
    }

    let indexes_vec: Vec<Index> = index_map
        .into_iter()
        .map(|(name, mut artists)| {
            artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            Index { name, artist: artists }
        })
        .collect();

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Indexes(Indexes {
        last_modified: scanner.last_scan_time() * 1000,
        ignored_articles: config.subsonic.ignored_articles.clone(),
        shortcut: vec![],
        index: indexes_vec,
        child: root_children,
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_music_directory(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    let dir = match child::Entity::find_by_id(id)
        .filter(child::Column::IsDir.eq(true))
        .one(*db)
        .await
    {
        Ok(Some(d)) => d,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Directory not found".into()),
                &params.f,
            )
        }
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    };

    let children = match child::Entity::find()
        .filter(child::Column::Parent.eq(dir.id.clone()))
        .order_by_desc(child::Column::IsDir)
        .order_by_asc(child::Column::Title)
        .all(*db)
        .await
    {
        Ok(c) => c,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to query directory".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Directory(Directory {
        id: dir.id,
        parent: if dir.parent.is_empty() {
            None
        } else {
            Some(dir.parent)
        },
        name: dir.title,
        starred: dir.starred,
        user_rating: Some(dir.user_rating),
        average_rating: Some(dir.average_rating),
        play_count: Some(dir.play_count),
        child: children.into_iter().map(map_child_to_subsonic).collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_genres(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    // Simplified genre query for now
    let genres = match genre::Entity::find().all(*db).await {
        Ok(g) => g,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to query genres".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Genres(Genres {
        genre: genres
            .into_iter()
            .map(|g| Genre {
                value: g.name,
                song_count: 0,  // TODO
                album_count: 0, // TODO
            })
            .collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artists(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
) -> impl IntoResponse {
    let artists = match artist::Entity::find().all(*db).await {
        Ok(a) => a,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to query artists".into()),
                &params.f,
            )
        }
    };

    let mut index_map: BTreeMap<String, Vec<ArtistID3>> = BTreeMap::new();
    for a in artists {
        if a.name.is_empty() {
            continue;
        }
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

    let index_vec: Vec<IndexID3> = index_map
        .into_iter()
        .map(|(name, mut artists)| {
            artists.sort_by(|a, b| a.name.cmp(&b.name));
            IndexID3 {
                name,
                artist: artists,
            }
        })
        .collect();

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Artists(ArtistsID3 {
        ignored_articles: "".into(),
        index: index_vec,
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_artist(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    let artist = match artist::Entity::find_by_id(id).one(*db).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Artist not found".into()),
                &params.f,
            )
        }
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    };

    // Fetch albums for this artist
    let albums = match album::Entity::find()
        .filter(album::Column::ArtistId.eq(artist.id.clone()))
        .all(*db)
        .await
    {
        Ok(al) => al,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch albums".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Artist(ArtistWithAlbumsID3 {
        artist: ArtistID3 {
            id: artist.id,
            name: artist.name,
            cover_art: (!artist.cover_art.is_empty()).then_some(artist.cover_art),
            artist_image_url: (!artist.artist_image_url.is_empty())
                .then_some(artist.artist_image_url),
            album_count: albums.len() as i32,
            starred: artist.starred,
            user_rating: Some(artist.user_rating),
            average_rating: Some(artist.average_rating),
        },
        album: albums
            .into_iter()
            .map(|al| AlbumID3 {
                id: al.id,
                name: al.name,
                artist: Some(al.artist),
                artist_id: Some(al.artist_id),
                cover_art: (!al.cover_art.is_empty()).then_some(al.cover_art),
                song_count: 0, // TODO
                duration: 0,   // TODO
                play_count: None,
                created: al.created,
                starred: al.starred,
                user_rating: Some(al.user_rating),
                average_rating: Some(al.average_rating),
                year: Some(al.year),
                genre: Some(al.genre),
            })
            .collect(),
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_album(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    let album = match album::Entity::find_by_id(id).one(*db).await {
        Ok(Some(al)) => al,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Album not found".into()),
                &params.f,
            )
        }
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    };

    let songs = match child::Entity::find()
        .filter(child::Column::AlbumId.eq(album.id.clone()))
        .order_by_asc(child::Column::DiscNumber)
        .order_by_asc(child::Column::Track)
        .all(*db)
        .await
    {
        Ok(s) => s,
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Failed to fetch songs".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Album(AlbumWithSongsID3 {
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
    }));

    send_response(resp, &params.f)
}

#[handler]
pub async fn get_song(
    db: Data<&DatabaseConnection>,
    params: Query<SubsonicParams>,
    query: Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = get_id_or_error!(query, params);

    let song = match child::Entity::find_by_id(id)
        .filter(child::Column::IsDir.eq(false))
        .one(*db)
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return send_response(
                SubsonicResponse::new_error(70, "Song not found".into()),
                &params.f,
            )
        }
        Err(_) => {
            return send_response(
                SubsonicResponse::new_error(0, "Database error".into()),
                &params.f,
            )
        }
    };

    let resp = SubsonicResponse::new_ok(SubsonicResponseBody::Song(map_child_to_subsonic(song)));

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
