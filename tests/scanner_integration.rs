//! Integration tests for the scanner + flusher pipeline.
//!
//! Generates a virtual library of 2000 songs (40 artists × 5 albums × 10 tracks)
//! and pushes them through the real flusher into an in-memory SQLite database
//! to verify that all rows (including foreign-key-dependent junction tables)
//! are inserted correctly.

use miko::config::Config;
use miko::models::{
    album, album_artist, album_genre, artist, child, genre, lyrics, song_artist, song_genre,
};
use miko::scanner::flusher;
use miko::scanner::seen;
use miko::scanner::types::{AlbumRelations, SongRelations, UpsertMessage};
use miko::scanner::utils;

use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ColumnTrait, ConnectionTrait, Database, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use std::sync::Arc;
use tokio::sync::mpsc;

// ─── constants ─────────────────────────────────────────────────

const NUM_ARTISTS: usize = 40;
const NUM_ALBUMS_PER_ARTIST: usize = 5; // 40 × 5 = 200 albums
const NUM_SONGS_PER_ALBUM: usize = 10; // 200 × 10 = 2000 songs
const NUM_GENRES: usize = 10;

// ─── helpers ───────────────────────────────────────────────────

fn artist_name(i: usize) -> String {
    format!("Artist {:04}", i)
}

fn album_name(artist_idx: usize, album_idx: usize) -> String {
    format!("Album {:04}-{:02}", artist_idx, album_idx)
}

fn genre_name(i: usize) -> String {
    format!("Genre {:02}", i)
}

fn song_title(artist_idx: usize, album_idx: usize, track: usize) -> String {
    format!("Track {:04}-{:02}-{:02}", artist_idx, album_idx, track)
}

/// Build all UpsertMessages that a full scan of 2000 songs would produce,
/// as if `process_task` + `build_artist/build_genre/build_album` ran for each file.
fn generate_virtual_library(folder_id: i32, folder_path: &str) -> Vec<UpsertMessage> {
    let mut all_batches: Vec<UpsertMessage> = Vec::new();
    let now = chrono::Utc::now();

    // Root folder directory entry
    let root_id = utils::generate_id(folder_path, folder_id, folder_path);
    all_batches.push(UpsertMessage::Batch(vec![
        UpsertMessage::Seen(root_id.clone()),
        UpsertMessage::Song(Box::new(child::ActiveModel {
            id: Set(root_id),
            parent: Set(None),
            is_dir: Set(true),
            title: Set("music".into()),
            path: Set(folder_path.into()),
            music_folder_id: Set(folder_id),
            content_type: Set(None),
            suffix: Set(None),
            transcoded_content_type: Set(None),
            transcoded_suffix: Set(None),
            album_id: Set(None),
            r#type: Set("directory".into()),
            track: Set(0),
            year: Set(0),
            disc_number: Set(0),
            duration: Set(0),
            bit_rate: Set(0),
            size: Set(0),
            is_video: Set(false),
            user_rating: Set(0),
            average_rating: Set(0.0),
            play_count: Set(0),
            ..Default::default()
        })),
    ]));

    for a_idx in 0..NUM_ARTISTS {
        let a_name = artist_name(a_idx);
        let a_id = utils::generate_artist_id(&a_name);

        // Artist directory
        let dir_path = format!("{}/{}", folder_path, a_name);
        let dir_id = utils::generate_id(&dir_path, folder_id, folder_path);
        let parent_id = utils::get_parent_id(&dir_path, folder_id, folder_path);
        all_batches.push(UpsertMessage::Batch(vec![
            UpsertMessage::Seen(dir_id.clone()),
            UpsertMessage::Song(Box::new(child::ActiveModel {
                id: Set(dir_id),
                parent: Set(parent_id),
                is_dir: Set(true),
                title: Set(a_name.clone()),
                path: Set(dir_path.clone()),
                music_folder_id: Set(folder_id),
                content_type: Set(None),
                suffix: Set(None),
                transcoded_content_type: Set(None),
                transcoded_suffix: Set(None),
                album_id: Set(None),
                r#type: Set("directory".into()),
                track: Set(0),
                year: Set(0),
                disc_number: Set(0),
                duration: Set(0),
                bit_rate: Set(0),
                size: Set(0),
                is_video: Set(false),
                user_rating: Set(0),
                average_rating: Set(0.0),
                play_count: Set(0),
                ..Default::default()
            })),
        ]));

        for al_idx in 0..NUM_ALBUMS_PER_ARTIST {
            let al_name = album_name(a_idx, al_idx);
            let al_id = utils::generate_album_id(&a_name, &al_name);
            let year = 2000 + (al_idx as i32);
            let g_name = genre_name((a_idx * NUM_ALBUMS_PER_ARTIST + al_idx) % NUM_GENRES);

            // Album directory
            let album_dir = format!("{}/{}", dir_path, al_name);
            let album_dir_id = utils::generate_id(&album_dir, folder_id, folder_path);
            let album_dir_parent = utils::get_parent_id(&album_dir, folder_id, folder_path);
            all_batches.push(UpsertMessage::Batch(vec![
                UpsertMessage::Seen(album_dir_id.clone()),
                UpsertMessage::Song(Box::new(child::ActiveModel {
                    id: Set(album_dir_id),
                    parent: Set(album_dir_parent),
                    is_dir: Set(true),
                    title: Set(al_name.clone()),
                    path: Set(album_dir.clone()),
                    music_folder_id: Set(folder_id),
                    content_type: Set(None),
                    suffix: Set(None),
                    transcoded_content_type: Set(None),
                    transcoded_suffix: Set(None),
                    album_id: Set(None),
                    r#type: Set("directory".into()),
                    track: Set(0),
                    year: Set(0),
                    disc_number: Set(0),
                    duration: Set(0),
                    bit_rate: Set(0),
                    size: Set(0),
                    is_video: Set(false),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    play_count: Set(0),
                    ..Default::default()
                })),
            ]));

            for track in 0..NUM_SONGS_PER_ALBUM {
                let title = song_title(a_idx, al_idx, track);
                let file_path = format!("{}/{}.mp3", album_dir, title);
                let song_id = utils::generate_id(&file_path, folder_id, folder_path);
                let parent_id = utils::get_parent_id(&file_path, folder_id, folder_path);

                let has_lyrics = track % 3 == 0;

                let mut batch = Vec::new();
                batch.push(UpsertMessage::Seen(song_id.clone()));

                // Artist
                batch.push(UpsertMessage::Artist(Box::new(artist::ActiveModel {
                    id: Set(a_id.clone()),
                    name: Set(a_name.clone()),
                    artist_image_url: Set(None),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));

                // Genre
                batch.push(UpsertMessage::Genre(Box::new(genre::ActiveModel {
                    name: Set(g_name.clone()),
                })));

                // Album
                batch.push(UpsertMessage::Album(Box::new(album::ActiveModel {
                    id: Set(al_id.clone()),
                    name: Set(al_name.clone()),
                    created: Set(now),
                    year: Set(year),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));

                // Album relations (artist on album)
                batch.push(UpsertMessage::Artist(Box::new(artist::ActiveModel {
                    id: Set(a_id.clone()),
                    name: Set(a_name.clone()),
                    artist_image_url: Set(None),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));
                batch.push(UpsertMessage::Genre(Box::new(genre::ActiveModel {
                    name: Set(g_name.clone()),
                })));
                batch.push(UpsertMessage::AlbumRelations(Box::new(AlbumRelations {
                    album_id: al_id.clone(),
                    artists: vec![a_id.clone()],
                    genres: vec![g_name.clone()],
                })));

                // Song
                batch.push(UpsertMessage::Song(Box::new(child::ActiveModel {
                    id: Set(song_id.clone()),
                    parent: Set(parent_id),
                    is_dir: Set(false),
                    title: Set(title.clone()),
                    path: Set(file_path),
                    size: Set(4_000_000),
                    suffix: Set(Some("mp3".into())),
                    content_type: Set(Some("audio/mp3".into())),
                    created: Set(Some(now)),
                    music_folder_id: Set(folder_id),
                    transcoded_content_type: Set(None),
                    transcoded_suffix: Set(None),
                    album_id: Set(Some(al_id.clone())),
                    r#type: Set("music".into()),
                    track: Set(track as i32 + 1),
                    year: Set(year),
                    disc_number: Set(1),
                    duration: Set(200 + track as i32),
                    bit_rate: Set(320),
                    is_video: Set(false),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    play_count: Set(0),
                    ..Default::default()
                })));

                // Song relations
                batch.push(UpsertMessage::SongRelations(Box::new(SongRelations {
                    song_id: song_id.clone(),
                    artists: vec![a_id.clone()],
                    genres: vec![g_name.clone()],
                    lyrics: if has_lyrics {
                        Some(format!("Lyrics for {}", title))
                    } else {
                        None
                    },
                })));

                all_batches.push(UpsertMessage::Batch(batch));
            }
        }
    }

    all_batches
}

/// Set up an in-memory SQLite DB with real migrations applied.
/// Uses `max_connections(1)` so all operations share the same in-memory DB.
async fn setup_db() -> sea_orm::DatabaseConnection {
    let mut opt = sea_orm::ConnectOptions::new("sqlite::memory:".to_string());
    opt.max_connections(1);
    let db = Database::connect(opt).await.unwrap();
    db.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

/// Insert the test music folder into the DB.
async fn insert_music_folder(db: &sea_orm::DatabaseConnection, id: i32, path: &str) {
    db.execute_unprepared(&format!(
        "INSERT INTO music_folders (id, path, name) VALUES ({}, '{}', 'Test')",
        id, path
    ))
    .await
    .unwrap();
}

// ─── tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn flusher_handles_2000_songs_without_fk_errors() {
    let db = setup_db().await;
    let folder_id = 1;
    let folder_path = "/music";
    insert_music_folder(&db, folder_id, folder_path).await;

    // Prepare seen tracker (as scan_all would)
    seen::SeenTracker::prepare(&db).await.unwrap();

    // Start the flusher
    let (tx, rx) = mpsc::channel(2000);
    let flusher_db = db.clone();
    let flusher_handle = tokio::spawn(async move {
        flusher::run_flusher(flusher_db, rx).await;
    });

    // Generate and send all 2000 songs + directories
    let messages = generate_virtual_library(folder_id, folder_path);
    let total_batches = messages.len();
    for msg in messages {
        tx.send(msg).await.unwrap();
    }

    // Request a flush and wait for ack
    let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
    tx.send(UpsertMessage::Flush(ack_tx)).await.unwrap();
    ack_rx.await.unwrap();

    // Close channel, flusher should exit cleanly
    drop(tx);
    flusher_handle.await.unwrap();

    // ─── Verify DB state ────────────────────────────────────
    let total_songs = NUM_ARTISTS * NUM_ALBUMS_PER_ARTIST * NUM_SONGS_PER_ALBUM;
    assert_eq!(total_songs, 2000);

    // Count songs (non-directory children)
    let song_count = child::Entity::count_songs(&db).await;
    assert_eq!(
        song_count, total_songs as i64,
        "Expected {} songs",
        total_songs
    );

    // Count directories: 1 root + 40 artist dirs + 200 album dirs = 241
    let dir_count: u64 = child::Entity::find()
        .filter(child::Column::IsDir.eq(true))
        .count(&db)
        .await
        .unwrap();
    let expected_dirs = 1 + NUM_ARTISTS + NUM_ARTISTS * NUM_ALBUMS_PER_ARTIST;
    assert_eq!(
        dir_count, expected_dirs as u64,
        "Expected {} directories",
        expected_dirs
    );

    // Count total children (songs + dirs)
    let total_children: u64 = child::Entity::find().count(&db).await.unwrap();
    assert_eq!(
        total_children,
        (total_songs + expected_dirs) as u64,
        "Expected {} total children",
        total_songs + expected_dirs,
    );

    // Count albums: 40 artists × 5 albums = 200
    let album_count: u64 = album::Entity::find().count(&db).await.unwrap();
    assert_eq!(album_count, (NUM_ARTISTS * NUM_ALBUMS_PER_ARTIST) as u64);

    // Count artists: 40
    let artist_count: u64 = artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(artist_count, NUM_ARTISTS as u64);

    // Count genres: 10
    let genre_count: u64 = genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(genre_count, NUM_GENRES as u64);

    // Count song_artists: one per song = 2000
    let sa_count: u64 = song_artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(sa_count, total_songs as u64);

    // Count song_genres: one per song = 2000
    let sg_count: u64 = song_genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(sg_count, total_songs as u64);

    // Count album_artists: one per album = 200
    let aa_count: u64 = album_artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(aa_count, (NUM_ARTISTS * NUM_ALBUMS_PER_ARTIST) as u64);

    // Count album_genres: one per album = 200
    let ag_count: u64 = album_genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(ag_count, (NUM_ARTISTS * NUM_ALBUMS_PER_ARTIST) as u64);

    // Count lyrics: every 3rd track per album = 4 per album × 200 = 800
    let lyrics_count: u64 = lyrics::Entity::find().count(&db).await.unwrap();
    let expected_lyrics = NUM_ARTISTS
        * NUM_ALBUMS_PER_ARTIST
        * (0..NUM_SONGS_PER_ALBUM).filter(|t| t % 3 == 0).count();
    assert_eq!(lyrics_count, expected_lyrics as u64);

    // Count seen IDs: should match total children
    let seen_count: u64 = seen::Entity::find().count(&db).await.unwrap();
    assert_eq!(seen_count, (total_songs + expected_dirs) as u64);

    eprintln!(
        "Integration test passed: {} batches, {} songs, {} dirs, {} albums, {} artists, {} genres, {} lyrics",
        total_batches, song_count, dir_count, album_count, artist_count, genre_count, lyrics_count,
    );
}

#[tokio::test]
async fn flusher_handles_2000_songs_then_prune_removes_missing() {
    let db = setup_db().await;
    let folder_id = 1;
    let folder_path = "/music";
    insert_music_folder(&db, folder_id, folder_path).await;

    // --- Phase 1: Full scan of 2000 songs ---
    seen::SeenTracker::prepare(&db).await.unwrap();

    let (tx, rx) = mpsc::channel(2000);
    let flusher_db = db.clone();
    let flusher_handle = tokio::spawn(async move {
        flusher::run_flusher(flusher_db, rx).await;
    });

    let messages = generate_virtual_library(folder_id, folder_path);
    for msg in messages {
        tx.send(msg).await.unwrap();
    }

    let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
    tx.send(UpsertMessage::Flush(ack_tx)).await.unwrap();
    ack_rx.await.unwrap();
    drop(tx);
    flusher_handle.await.unwrap();

    // --- Phase 2: Simulate a scan where half the artists are gone ---
    seen::SeenTracker::prepare(&db).await.unwrap();

    let (tx2, rx2) = mpsc::channel(2000);
    let flusher_db2 = db.clone();
    let flusher_handle2 = tokio::spawn(async move {
        flusher::run_flusher(flusher_db2, rx2).await;
    });

    let half_artists = NUM_ARTISTS / 2;
    let now = chrono::Utc::now();

    // Root dir
    let root_id = utils::generate_id(folder_path, folder_id, folder_path);
    tx2.send(UpsertMessage::Batch(vec![
        UpsertMessage::Seen(root_id.clone()),
        UpsertMessage::Song(Box::new(child::ActiveModel {
            id: Set(root_id),
            parent: Set(None),
            is_dir: Set(true),
            title: Set("music".into()),
            path: Set(folder_path.into()),
            music_folder_id: Set(folder_id),
            content_type: Set(None),
            suffix: Set(None),
            transcoded_content_type: Set(None),
            transcoded_suffix: Set(None),
            album_id: Set(None),
            r#type: Set("directory".into()),
            track: Set(0),
            year: Set(0),
            disc_number: Set(0),
            duration: Set(0),
            bit_rate: Set(0),
            size: Set(0),
            is_video: Set(false),
            user_rating: Set(0),
            average_rating: Set(0.0),
            play_count: Set(0),
            ..Default::default()
        })),
    ]))
    .await
    .unwrap();

    for a_idx in 0..half_artists {
        let a_name = artist_name(a_idx);
        let a_id = utils::generate_artist_id(&a_name);
        let dir_path = format!("{}/{}", folder_path, a_name);
        let dir_id = utils::generate_id(&dir_path, folder_id, folder_path);
        let parent_id = utils::get_parent_id(&dir_path, folder_id, folder_path);

        tx2.send(UpsertMessage::Batch(vec![
            UpsertMessage::Seen(dir_id.clone()),
            UpsertMessage::Song(Box::new(child::ActiveModel {
                id: Set(dir_id),
                parent: Set(parent_id),
                is_dir: Set(true),
                title: Set(a_name.clone()),
                path: Set(dir_path.clone()),
                music_folder_id: Set(folder_id),
                content_type: Set(None),
                suffix: Set(None),
                transcoded_content_type: Set(None),
                transcoded_suffix: Set(None),
                album_id: Set(None),
                r#type: Set("directory".into()),
                track: Set(0),
                year: Set(0),
                disc_number: Set(0),
                duration: Set(0),
                bit_rate: Set(0),
                size: Set(0),
                is_video: Set(false),
                user_rating: Set(0),
                average_rating: Set(0.0),
                play_count: Set(0),
                ..Default::default()
            })),
        ]))
        .await
        .unwrap();

        for al_idx in 0..NUM_ALBUMS_PER_ARTIST {
            let al_name = album_name(a_idx, al_idx);
            let al_id = utils::generate_album_id(&a_name, &al_name);
            let year = 2000 + al_idx as i32;
            let g_name = genre_name((a_idx * NUM_ALBUMS_PER_ARTIST + al_idx) % NUM_GENRES);

            let album_dir = format!("{}/{}", dir_path, al_name);
            let album_dir_id = utils::generate_id(&album_dir, folder_id, folder_path);
            let album_dir_parent = utils::get_parent_id(&album_dir, folder_id, folder_path);

            tx2.send(UpsertMessage::Batch(vec![
                UpsertMessage::Seen(album_dir_id.clone()),
                UpsertMessage::Song(Box::new(child::ActiveModel {
                    id: Set(album_dir_id),
                    parent: Set(album_dir_parent),
                    is_dir: Set(true),
                    title: Set(al_name.clone()),
                    path: Set(album_dir.clone()),
                    music_folder_id: Set(folder_id),
                    content_type: Set(None),
                    suffix: Set(None),
                    transcoded_content_type: Set(None),
                    transcoded_suffix: Set(None),
                    album_id: Set(None),
                    r#type: Set("directory".into()),
                    track: Set(0),
                    year: Set(0),
                    disc_number: Set(0),
                    duration: Set(0),
                    bit_rate: Set(0),
                    size: Set(0),
                    is_video: Set(false),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    play_count: Set(0),
                    ..Default::default()
                })),
            ]))
            .await
            .unwrap();

            for track in 0..NUM_SONGS_PER_ALBUM {
                let title = song_title(a_idx, al_idx, track);
                let file_path = format!("{}/{}.mp3", album_dir, title);
                let song_id = utils::generate_id(&file_path, folder_id, folder_path);
                let parent_id = utils::get_parent_id(&file_path, folder_id, folder_path);
                let has_lyrics = track % 3 == 0;

                let mut batch = Vec::new();
                batch.push(UpsertMessage::Seen(song_id.clone()));
                batch.push(UpsertMessage::Artist(Box::new(artist::ActiveModel {
                    id: Set(a_id.clone()),
                    name: Set(a_name.clone()),
                    artist_image_url: Set(None),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));
                batch.push(UpsertMessage::Genre(Box::new(genre::ActiveModel {
                    name: Set(g_name.clone()),
                })));
                batch.push(UpsertMessage::Album(Box::new(album::ActiveModel {
                    id: Set(al_id.clone()),
                    name: Set(al_name.clone()),
                    created: Set(now),
                    year: Set(year),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));
                batch.push(UpsertMessage::Artist(Box::new(artist::ActiveModel {
                    id: Set(a_id.clone()),
                    name: Set(a_name.clone()),
                    artist_image_url: Set(None),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    ..Default::default()
                })));
                batch.push(UpsertMessage::Genre(Box::new(genre::ActiveModel {
                    name: Set(g_name.clone()),
                })));
                batch.push(UpsertMessage::AlbumRelations(Box::new(AlbumRelations {
                    album_id: al_id.clone(),
                    artists: vec![a_id.clone()],
                    genres: vec![g_name.clone()],
                })));
                batch.push(UpsertMessage::Song(Box::new(child::ActiveModel {
                    id: Set(song_id.clone()),
                    parent: Set(parent_id),
                    is_dir: Set(false),
                    title: Set(title.clone()),
                    path: Set(file_path),
                    size: Set(4_000_000),
                    suffix: Set(Some("mp3".into())),
                    content_type: Set(Some("audio/mp3".into())),
                    created: Set(Some(now)),
                    music_folder_id: Set(folder_id),
                    transcoded_content_type: Set(None),
                    transcoded_suffix: Set(None),
                    album_id: Set(Some(al_id.clone())),
                    r#type: Set("music".into()),
                    track: Set(track as i32 + 1),
                    year: Set(year),
                    disc_number: Set(1),
                    duration: Set(200 + track as i32),
                    bit_rate: Set(320),
                    is_video: Set(false),
                    user_rating: Set(0),
                    average_rating: Set(0.0),
                    play_count: Set(0),
                    ..Default::default()
                })));
                batch.push(UpsertMessage::SongRelations(Box::new(SongRelations {
                    song_id: song_id.clone(),
                    artists: vec![a_id.clone()],
                    genres: vec![g_name.clone()],
                    lyrics: if has_lyrics {
                        Some(format!("Lyrics for {}", title))
                    } else {
                        None
                    },
                })));
                tx2.send(UpsertMessage::Batch(batch)).await.unwrap();
            }
        }
    }

    let (ack_tx2, ack_rx2) = tokio::sync::oneshot::channel();
    tx2.send(UpsertMessage::Flush(ack_tx2)).await.unwrap();
    ack_rx2.await.unwrap();
    drop(tx2);
    flusher_handle2.await.unwrap();

    // --- Phase 3: Prune (simulates scanner.prune()) ---
    db.execute_unprepared(
        "DELETE FROM lyrics WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = lyrics.song_id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM song_artists WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_artists.song_id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM song_genres WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = song_genres.song_id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM children WHERE NOT EXISTS (SELECT 1 FROM _scanner_seen WHERE _scanner_seen.id = children.id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM album_artists WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = album_artists.album_id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM album_genres WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = album_genres.album_id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM albums WHERE NOT EXISTS (SELECT 1 FROM children WHERE children.album_id = albums.id)",
    ).await.unwrap();
    db.execute_unprepared(
        "DELETE FROM artists \
         WHERE NOT EXISTS (SELECT 1 FROM song_artists WHERE song_artists.artist_id = artists.id) \
         AND NOT EXISTS (SELECT 1 FROM album_artists WHERE album_artists.artist_id = artists.id)",
    )
    .await
    .unwrap();
    db.execute_unprepared(
        "DELETE FROM genres \
         WHERE NOT EXISTS (SELECT 1 FROM album_genres WHERE album_genres.genre_name = genres.name) \
         AND NOT EXISTS (SELECT 1 FROM song_genres WHERE song_genres.genre_name = genres.name)",
    )
    .await
    .unwrap();

    // --- Phase 4: Verify pruned state ---
    let half_songs = half_artists * NUM_ALBUMS_PER_ARTIST * NUM_SONGS_PER_ALBUM;
    let half_dirs = 1 + half_artists + half_artists * NUM_ALBUMS_PER_ARTIST;

    let song_count = child::Entity::count_songs(&db).await;
    assert_eq!(
        song_count, half_songs as i64,
        "Expected {} songs after prune",
        half_songs
    );

    let dir_count: u64 = child::Entity::find()
        .filter(child::Column::IsDir.eq(true))
        .count(&db)
        .await
        .unwrap();
    assert_eq!(
        dir_count, half_dirs as u64,
        "Expected {} dirs after prune",
        half_dirs
    );

    let album_count: u64 = album::Entity::find().count(&db).await.unwrap();
    let half_albums = half_artists * NUM_ALBUMS_PER_ARTIST;
    assert_eq!(
        album_count, half_albums as u64,
        "Expected {} albums after prune",
        half_albums
    );

    let artist_count: u64 = artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(
        artist_count, half_artists as u64,
        "Expected {} artists after prune",
        half_artists
    );

    // All 10 genres still referenced by remaining songs (both halves cover 0..9 mod 10)
    let genre_count: u64 = genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(
        genre_count, NUM_GENRES as u64,
        "All genres still referenced by remaining songs"
    );

    let sa_count: u64 = song_artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(sa_count, half_songs as u64);

    let sg_count: u64 = song_genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(sg_count, half_songs as u64);
}
