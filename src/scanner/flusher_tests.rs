use super::*;
use crate::models::{album, artist, child, genre};
use crate::scanner::types::{AlbumRelations, SongRelations, UpsertMessage};
use sea_orm::Set;

// Re-export for do_flush_cycle tests
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, EntityTrait, PaginatorTrait};

// ─── helpers ───────────────────────────────────────────────────

fn make_artist(id: &str) -> UpsertMessage {
    UpsertMessage::Artist(Box::new(artist::ActiveModel {
        id: Set(id.to_string()),
        name: Set(id.to_string()),
        artist_image_url: Set(None),
        average_rating: Set(0.0),
        ..Default::default()
    }))
}

fn make_genre(name: &str) -> UpsertMessage {
    UpsertMessage::Genre(Box::new(genre::ActiveModel {
        name: Set(name.to_string()),
    }))
}

fn make_album(id: &str) -> UpsertMessage {
    UpsertMessage::Album(Box::new(album::ActiveModel {
        id: Set(id.to_string()),
        name: Set(id.to_string()),
        created: Set(chrono::Utc::now()),
        year: Set(2024),
        average_rating: Set(0.0),
        ..Default::default()
    }))
}

fn make_song(id: &str, path: &str, is_dir: bool) -> UpsertMessage {
    UpsertMessage::Song(Box::new(child::ActiveModel {
        id: Set(id.to_string()),
        parent: Set(None),
        is_dir: Set(is_dir),
        title: Set(id.to_string()),
        path: Set(path.to_string()),
        music_folder_id: Set(1),
        content_type: Set(None),
        suffix: Set(None),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(None),
        r#type: Set("music".to_string()),
        track: Set(0),
        year: Set(0),
        disc_number: Set(0),
        duration: Set(0),
        bit_rate: Set(0),
        size: Set(0),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    }))
}

fn make_song_relations(song_id: &str) -> UpsertMessage {
    UpsertMessage::SongRelations(Box::new(SongRelations {
        song_id: song_id.to_string(),
        artists: vec!["a1".into()],
        genres: vec!["rock".into()],
        lyrics: None,
    }))
}

fn make_album_relations(album_id: &str) -> UpsertMessage {
    UpsertMessage::AlbumRelations(Box::new(AlbumRelations {
        album_id: album_id.to_string(),
        artists: vec!["a1".into()],
        genres: vec!["rock".into()],
    }))
}

fn new_buffers() -> (
    Vec<artist::ActiveModel>,
    Vec<album::ActiveModel>,
    Vec<genre::ActiveModel>,
    Vec<child::ActiveModel>,
    Vec<SongRelations>,
    Vec<AlbumRelations>,
    Vec<String>,
    bool,
    Option<tokio::sync::oneshot::Sender<()>>,
) {
    (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        false,
        None,
    )
}

// ─── dispatch tests ────────────────────────────────────────────

#[test]
fn dispatch_routes_artist() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_artist("a1"),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(artists.len(), 1);
    assert_eq!(*artists[0].id.as_ref(), "a1");
    assert!(albums.is_empty());
}

#[test]
fn dispatch_routes_genre() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_genre("rock"),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(genres.len(), 1);
    assert_eq!(*genres[0].name.as_ref(), "rock");
}

#[test]
fn dispatch_routes_album() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_album("al1"),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(albums.len(), 1);
    assert_eq!(*albums[0].id.as_ref(), "al1");
}

#[test]
fn dispatch_routes_song() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_song("s1", "/music/a.mp3", false),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(songs.len(), 1);
    assert_eq!(*songs[0].id.as_ref(), "s1");
}

#[test]
fn dispatch_routes_seen() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        UpsertMessage::Seen("id1".into()),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(seen, vec!["id1".to_string()]);
}

#[test]
fn dispatch_routes_song_relations() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_song_relations("s1"),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(sr.len(), 1);
    assert_eq!(sr[0].song_id, "s1");
}

#[test]
fn dispatch_routes_album_relations() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    dispatch(
        make_album_relations("al1"),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(ar.len(), 1);
    assert_eq!(ar[0].album_id, "al1");
}

#[test]
fn dispatch_flush_sets_flag() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    let (tx, _rx) = tokio::sync::oneshot::channel();
    dispatch(
        UpsertMessage::Flush(tx),
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert!(ff);
    assert!(fa.is_some());
}

// ─── batch flattening tests ────────────────────────────────────

#[test]
fn dispatch_batch_flattens_messages() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    let batch = UpsertMessage::Batch(vec![
        make_artist("a1"),
        make_genre("rock"),
        make_album("al1"),
        make_song("s1", "/music/a.mp3", false),
        make_song_relations("s1"),
        make_album_relations("al1"),
        UpsertMessage::Seen("id1".into()),
    ]);
    dispatch(
        batch,
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(artists.len(), 1);
    assert_eq!(genres.len(), 1);
    assert_eq!(albums.len(), 1);
    assert_eq!(songs.len(), 1);
    assert_eq!(sr.len(), 1);
    assert_eq!(ar.len(), 1);
    assert_eq!(seen.len(), 1);
}

#[test]
fn dispatch_nested_batch() {
    let (mut artists, mut albums, mut genres, mut songs, mut sr, mut ar, mut seen, mut ff, mut fa) =
        new_buffers();
    let inner = UpsertMessage::Batch(vec![make_artist("a1"), make_artist("a2")]);
    let outer = UpsertMessage::Batch(vec![inner, make_artist("a3")]);
    dispatch(
        outer,
        &mut artists,
        &mut albums,
        &mut genres,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
        &mut ff,
        &mut fa,
    );
    assert_eq!(artists.len(), 3);
}

// ─── sort_songs_for_insert tests ───────────────────────────────

fn make_child_active(id: &str, path: &str, is_dir: bool) -> child::ActiveModel {
    child::ActiveModel {
        id: Set(id.to_string()),
        parent: Set(None),
        is_dir: Set(is_dir),
        title: Set(id.to_string()),
        path: Set(path.to_string()),
        music_folder_id: Set(1),
        content_type: Set(None),
        suffix: Set(None),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(None),
        r#type: Set("music".to_string()),
        track: Set(0),
        year: Set(0),
        disc_number: Set(0),
        duration: Set(0),
        bit_rate: Set(0),
        size: Set(0),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    }
}

#[test]
fn sort_directories_before_files() {
    let mut items = vec![
        make_child_active("f1", "/music/Artist/Album/song.mp3", false),
        make_child_active("d1", "/music/Artist/Album", true),
        make_child_active("f2", "/music/Artist/Album/other.mp3", false),
        make_child_active("d2", "/music/Artist", true),
    ];
    sort_songs_for_insert(&mut items);

    // First two should be directories
    assert!(*items[0].is_dir.as_ref());
    assert!(*items[1].is_dir.as_ref());
    // Last two should be files
    assert!(!*items[2].is_dir.as_ref());
    assert!(!*items[3].is_dir.as_ref());
}

#[test]
fn sort_directories_by_path_depth() {
    let mut items = vec![
        make_child_active("d3", "/music/Artist/Album/Disc1", true),
        make_child_active("d1", "/music/Artist", true),
        make_child_active("d2", "/music/Artist/Album", true),
    ];
    sort_songs_for_insert(&mut items);

    let paths: Vec<&str> = items.iter().map(|i| i.path.as_ref().as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "/music/Artist",
            "/music/Artist/Album",
            "/music/Artist/Album/Disc1",
        ]
    );
}

#[test]
fn sort_parent_directory_before_child_file() {
    let mut items = vec![
        make_child_active("f1", "/music/Artist/Album/01 Track.mp3", false),
        make_child_active("d1", "/music/Artist/Album", true),
        make_child_active("d0", "/music/Artist", true),
    ];
    sort_songs_for_insert(&mut items);

    // Directories first, then file
    assert_eq!(items[0].id.as_ref(), "d0");
    assert_eq!(items[1].id.as_ref(), "d1");
    assert_eq!(items[2].id.as_ref(), "f1");
}

#[test]
fn sort_empty_vec_is_noop() {
    let mut items: Vec<child::ActiveModel> = vec![];
    sort_songs_for_insert(&mut items);
    assert!(items.is_empty());
}

// ─── should_flush threshold tests ──────────────────────────────

/// Simulates the should_flush logic from run_flusher.
fn should_flush(
    artists: &[artist::ActiveModel],
    genres: &[genre::ActiveModel],
    albums: &[album::ActiveModel],
    songs: &[child::ActiveModel],
    song_relations: &[SongRelations],
    album_relations: &[AlbumRelations],
    seen_ids: &[String],
    overdue: bool,
    force_flush: bool,
) -> bool {
    let any_threshold = artists.len() >= 100
        || genres.len() >= 50
        || albums.len() >= 100
        || songs.len() >= 100
        || song_relations.len() >= 100
        || album_relations.len() >= 100
        || seen_ids.len() >= 500;

    let has_data = !artists.is_empty()
        || !genres.is_empty()
        || !albums.is_empty()
        || !songs.is_empty()
        || !song_relations.is_empty()
        || !album_relations.is_empty()
        || !seen_ids.is_empty();

    any_threshold || (overdue && has_data) || force_flush
}

#[test]
fn no_flush_when_empty_and_not_overdue() {
    assert!(!should_flush(
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false
    ));
}

#[test]
fn no_flush_when_overdue_but_empty() {
    assert!(!should_flush(
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        true,
        false
    ));
}

#[test]
fn flush_on_force() {
    assert!(should_flush(&[], &[], &[], &[], &[], &[], &[], false, true));
}

#[test]
fn flush_when_overdue_with_data() {
    let artists = vec![artist::ActiveModel {
        id: Set("a1".into()),
        name: Set("A1".into()),
        artist_image_url: Set(None),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    assert!(should_flush(
        &artists,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        true,
        false
    ));
}

#[test]
fn flush_when_artist_threshold_reached() {
    let artists: Vec<_> = (0..100)
        .map(|i| artist::ActiveModel {
            id: Set(format!("a{}", i)),
            name: Set(format!("Artist {}", i)),
            artist_image_url: Set(None),
            average_rating: Set(0.0),
            ..Default::default()
        })
        .collect();
    assert!(should_flush(
        &artists,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false
    ));
}

#[test]
fn flush_when_genre_threshold_reached() {
    let genres: Vec<_> = (0..50)
        .map(|i| genre::ActiveModel {
            name: Set(format!("Genre {}", i)),
        })
        .collect();
    assert!(should_flush(
        &[],
        &genres,
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false
    ));
}

#[test]
fn flush_when_song_threshold_reached() {
    let songs: Vec<_> = (0..100)
        .map(|i| make_child_active(&format!("s{}", i), &format!("/music/{}.mp3", i), false))
        .collect();
    assert!(should_flush(
        &[],
        &[],
        &[],
        &songs,
        &[],
        &[],
        &[],
        false,
        false
    ));
}

#[test]
fn flush_when_seen_threshold_reached() {
    let seen: Vec<String> = (0..500).map(|i| format!("id{}", i)).collect();
    assert!(should_flush(
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &seen,
        false,
        false
    ));
}

#[test]
fn no_flush_below_all_thresholds_not_overdue() {
    let artists: Vec<_> = (0..99)
        .map(|i| artist::ActiveModel {
            id: Set(format!("a{}", i)),
            name: Set(format!("Artist {}", i)),
            artist_image_url: Set(None),
            average_rating: Set(0.0),
            ..Default::default()
        })
        .collect();
    let genres: Vec<_> = (0..49)
        .map(|i| genre::ActiveModel {
            name: Set(format!("Genre {}", i)),
        })
        .collect();
    // Below all thresholds and not overdue → no flush
    assert!(!should_flush(
        &artists,
        &genres,
        &[],
        &[],
        &[],
        &[],
        &[],
        false,
        false
    ));
}

// ─── do_flush_cycle DB tests ───────────────────────────────────

async fn test_db() -> sea_orm::DatabaseConnection {
    let mut opt = sea_orm::ConnectOptions::new("sqlite::memory:".to_string());
    opt.max_connections(1);
    let db = Database::connect(opt).await.unwrap();
    db.execute_unprepared("PRAGMA foreign_keys = ON")
        .await
        .unwrap();
    Migrator::up(&db, None).await.unwrap();
    db.execute_unprepared(
        "INSERT INTO music_folders (id, path, name) VALUES (1, '/music', 'Test')",
    )
    .await
    .unwrap();
    // Create the _scanner_seen table (normally done by SeenTracker::prepare)
    crate::scanner::seen::SeenTracker::prepare(&db)
        .await
        .unwrap();
    db
}

/// A song referencing an album that is created in the same flush cycle
/// should succeed thanks to deferred FK constraints.
#[tokio::test]
async fn flush_cycle_song_references_album_in_same_batch() {
    let db = test_db().await;
    let now = chrono::Utc::now();

    let mut artists = vec![artist::ActiveModel {
        id: Set("a1".into()),
        name: Set("Artist One".into()),
        artist_image_url: Set(None),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    let mut genres = vec![genre::ActiveModel {
        name: Set("Rock".into()),
    }];
    let mut albums = vec![album::ActiveModel {
        id: Set("al1".into()),
        name: Set("Album One".into()),
        created: Set(now),
        year: Set(2024),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    let mut songs = vec![child::ActiveModel {
        id: Set("s1".into()),
        parent: Set(None),
        is_dir: Set(false),
        title: Set("Song One".into()),
        path: Set("/music/song1.mp3".into()),
        music_folder_id: Set(1),
        content_type: Set(Some("audio/mp3".into())),
        suffix: Set(Some("mp3".into())),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(Some("al1".into())), // references album in same batch
        r#type: Set("music".into()),
        track: Set(1),
        year: Set(2024),
        disc_number: Set(1),
        duration: Set(200),
        bit_rate: Set(320),
        size: Set(4_000_000),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    }];
    let mut song_relations = vec![SongRelations {
        song_id: "s1".into(),
        artists: vec!["a1".into()],
        genres: vec!["Rock".into()],
        lyrics: Some("Hello world".into()),
    }];
    let mut album_relations = vec![AlbumRelations {
        album_id: "al1".into(),
        artists: vec!["a1".into()],
        genres: vec!["Rock".into()],
    }];
    let mut seen_ids = vec!["s1".into()];

    do_flush_cycle(
        &db,
        &mut artists,
        &mut genres,
        &mut albums,
        &mut songs,
        &mut song_relations,
        &mut album_relations,
        &mut seen_ids,
    )
    .await
    .expect("flush_cycle should succeed with deferred FKs");

    // All buffers should be drained
    assert!(artists.is_empty());
    assert!(genres.is_empty());
    assert!(albums.is_empty());
    assert!(songs.is_empty());
    assert!(song_relations.is_empty());
    assert!(album_relations.is_empty());
    assert!(seen_ids.is_empty());

    // Verify rows
    let song_count: u64 = child::Entity::find().count(&db).await.unwrap();
    assert_eq!(song_count, 1);
    let album_count: u64 = album::Entity::find().count(&db).await.unwrap();
    assert_eq!(album_count, 1);
    let artist_count: u64 = artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(artist_count, 1);
}

/// A directory and its child file in the same flush cycle should
/// succeed (self-referencing parent FK on children table).
#[tokio::test]
async fn flush_cycle_parent_child_directory_in_same_batch() {
    let db = test_db().await;

    let mut artists = vec![];
    let mut genres = vec![];
    let mut albums = vec![];
    let mut songs = vec![
        child::ActiveModel {
            id: Set("dir1".into()),
            parent: Set(None),
            is_dir: Set(true),
            title: Set("ArtistDir".into()),
            path: Set("/music/ArtistDir".into()),
            music_folder_id: Set(1),
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
            average_rating: Set(0.0),
            play_count: Set(0),
            ..Default::default()
        },
        child::ActiveModel {
            id: Set("file1".into()),
            parent: Set(Some("dir1".into())), // references dir in same batch
            is_dir: Set(false),
            title: Set("Track 01".into()),
            path: Set("/music/ArtistDir/track01.mp3".into()),
            music_folder_id: Set(1),
            content_type: Set(Some("audio/mp3".into())),
            suffix: Set(Some("mp3".into())),
            transcoded_content_type: Set(None),
            transcoded_suffix: Set(None),
            album_id: Set(None),
            r#type: Set("music".into()),
            track: Set(1),
            year: Set(0),
            disc_number: Set(0),
            duration: Set(180),
            bit_rate: Set(256),
            size: Set(3_000_000),
            is_video: Set(false),
            average_rating: Set(0.0),
            play_count: Set(0),
            ..Default::default()
        },
    ];
    let mut song_relations = vec![];
    let mut album_relations = vec![];
    let mut seen_ids = vec!["dir1".into(), "file1".into()];

    do_flush_cycle(
        &db,
        &mut artists,
        &mut genres,
        &mut albums,
        &mut songs,
        &mut song_relations,
        &mut album_relations,
        &mut seen_ids,
    )
    .await
    .expect("flush_cycle should handle parent-child in same batch");

    let count: u64 = child::Entity::find().count(&db).await.unwrap();
    assert_eq!(count, 2);
}

/// Song relations referencing songs+artists+genres all created
/// in the same flush cycle should succeed.
#[tokio::test]
async fn flush_cycle_relations_reference_entities_in_same_batch() {
    let db = test_db().await;
    let now = chrono::Utc::now();

    let mut artists = vec![
        artist::ActiveModel {
            id: Set("a1".into()),
            name: Set("Artist A".into()),
            artist_image_url: Set(None),
            average_rating: Set(0.0),
            ..Default::default()
        },
        artist::ActiveModel {
            id: Set("a2".into()),
            name: Set("Artist B".into()),
            artist_image_url: Set(None),
            average_rating: Set(0.0),
            ..Default::default()
        },
    ];
    let mut genres = vec![
        genre::ActiveModel {
            name: Set("Rock".into()),
        },
        genre::ActiveModel {
            name: Set("Pop".into()),
        },
    ];
    let mut albums = vec![album::ActiveModel {
        id: Set("al1".into()),
        name: Set("Collaboration".into()),
        created: Set(now),
        year: Set(2024),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    let mut songs = vec![child::ActiveModel {
        id: Set("s1".into()),
        parent: Set(None),
        is_dir: Set(false),
        title: Set("Duet".into()),
        path: Set("/music/duet.mp3".into()),
        music_folder_id: Set(1),
        content_type: Set(Some("audio/mp3".into())),
        suffix: Set(Some("mp3".into())),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(Some("al1".into())),
        r#type: Set("music".into()),
        track: Set(1),
        year: Set(2024),
        disc_number: Set(1),
        duration: Set(240),
        bit_rate: Set(320),
        size: Set(5_000_000),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    }];
    // Song has two artists and two genres
    let mut song_relations = vec![SongRelations {
        song_id: "s1".into(),
        artists: vec!["a1".into(), "a2".into()],
        genres: vec!["Rock".into(), "Pop".into()],
        lyrics: None,
    }];
    let mut album_relations = vec![AlbumRelations {
        album_id: "al1".into(),
        artists: vec!["a1".into(), "a2".into()],
        genres: vec!["Rock".into(), "Pop".into()],
    }];
    let mut seen_ids = vec!["s1".into()];

    do_flush_cycle(
        &db,
        &mut artists,
        &mut genres,
        &mut albums,
        &mut songs,
        &mut song_relations,
        &mut album_relations,
        &mut seen_ids,
    )
    .await
    .expect("flush_cycle should handle multi-artist/genre relations");

    use crate::models::{album_artist, album_genre, lyrics, song_artist, song_genre};
    let sa: u64 = song_artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(sa, 2, "song should have 2 artists");
    let sg: u64 = song_genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(sg, 2, "song should have 2 genres");
    let aa: u64 = album_artist::Entity::find().count(&db).await.unwrap();
    assert_eq!(aa, 2, "album should have 2 artists");
    let ag: u64 = album_genre::Entity::find().count(&db).await.unwrap();
    assert_eq!(ag, 2, "album should have 2 genres");
    let ly: u64 = lyrics::Entity::find().count(&db).await.unwrap();
    assert_eq!(ly, 0, "no lyrics expected");
}

/// Two consecutive flush cycles should work correctly:
/// the second one can update/replace data from the first.
#[tokio::test]
async fn flush_cycle_two_consecutive_cycles() {
    let db = test_db().await;
    let now = chrono::Utc::now();

    // Cycle 1: insert artist + album + song
    let mut artists = vec![artist::ActiveModel {
        id: Set("a1".into()),
        name: Set("Artist".into()),
        artist_image_url: Set(None),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    let mut genres = vec![genre::ActiveModel {
        name: Set("Rock".into()),
    }];
    let mut albums = vec![album::ActiveModel {
        id: Set("al1".into()),
        name: Set("Album".into()),
        created: Set(now),
        year: Set(2020),
        average_rating: Set(0.0),
        ..Default::default()
    }];
    let mut songs = vec![child::ActiveModel {
        id: Set("s1".into()),
        parent: Set(None),
        is_dir: Set(false),
        title: Set("Old Title".into()),
        path: Set("/music/song.mp3".into()),
        music_folder_id: Set(1),
        content_type: Set(Some("audio/mp3".into())),
        suffix: Set(Some("mp3".into())),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(Some("al1".into())),
        r#type: Set("music".into()),
        track: Set(1),
        year: Set(2020),
        disc_number: Set(1),
        duration: Set(200),
        bit_rate: Set(320),
        size: Set(4_000_000),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    }];
    let mut sr = vec![SongRelations {
        song_id: "s1".into(),
        artists: vec!["a1".into()],
        genres: vec!["Rock".into()],
        lyrics: Some("Old lyrics".into()),
    }];
    let mut ar = vec![];
    let mut seen = vec!["s1".into()];

    do_flush_cycle(
        &db,
        &mut artists,
        &mut genres,
        &mut albums,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
    )
    .await
    .unwrap();

    // Cycle 2: update same song with new title and new lyrics
    artists.push(artist::ActiveModel {
        id: Set("a1".into()),
        name: Set("Artist".into()),
        artist_image_url: Set(None),
        average_rating: Set(0.0),
        ..Default::default()
    });
    genres.push(genre::ActiveModel {
        name: Set("Rock".into()),
    });
    albums.push(album::ActiveModel {
        id: Set("al1".into()),
        name: Set("Album".into()),
        created: Set(now),
        year: Set(2024), // updated year
        average_rating: Set(0.0),
        ..Default::default()
    });
    songs.push(child::ActiveModel {
        id: Set("s1".into()),
        parent: Set(None),
        is_dir: Set(false),
        title: Set("New Title".into()), // updated title
        path: Set("/music/song.mp3".into()),
        music_folder_id: Set(1),
        content_type: Set(Some("audio/mp3".into())),
        suffix: Set(Some("mp3".into())),
        transcoded_content_type: Set(None),
        transcoded_suffix: Set(None),
        album_id: Set(Some("al1".into())),
        r#type: Set("music".into()),
        track: Set(1),
        year: Set(2024),
        disc_number: Set(1),
        duration: Set(200),
        bit_rate: Set(320),
        size: Set(4_000_000),
        is_video: Set(false),
        average_rating: Set(0.0),
        play_count: Set(0),
        ..Default::default()
    });
    sr.push(SongRelations {
        song_id: "s1".into(),
        artists: vec!["a1".into()],
        genres: vec!["Rock".into()],
        lyrics: Some("New lyrics".into()),
    });

    do_flush_cycle(
        &db,
        &mut artists,
        &mut genres,
        &mut albums,
        &mut songs,
        &mut sr,
        &mut ar,
        &mut seen,
    )
    .await
    .unwrap();

    // Should still be 1 song, 1 album, 1 artist
    let count: u64 = child::Entity::find().count(&db).await.unwrap();
    assert_eq!(count, 1);
    let album_count: u64 = album::Entity::find().count(&db).await.unwrap();
    assert_eq!(album_count, 1);

    // Verify the title was updated
    let song = child::Entity::find_by_id("s1")
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(song.title, "New Title");

    // Verify album year was updated
    let alb = album::Entity::find_by_id("al1")
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(alb.year, 2024);
}

/// Empty flush cycle should be a no-op and not error.
#[tokio::test]
async fn flush_cycle_empty_is_noop() {
    let db = test_db().await;
    let mut a = vec![];
    let mut g = vec![];
    let mut al = vec![];
    let mut s = vec![];
    let mut sr = vec![];
    let mut ar = vec![];
    let mut si = vec![];

    do_flush_cycle(
        &db, &mut a, &mut g, &mut al, &mut s, &mut sr, &mut ar, &mut si,
    )
    .await
    .expect("empty flush should succeed");
}
