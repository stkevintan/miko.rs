use super::*;
use crate::models::{album, artist, child, genre};
use crate::scanner::types::{AlbumRelations, SongRelations, UpsertMessage};
use sea_orm::Set;

// ─── helpers ───────────────────────────────────────────────────

fn make_artist(id: &str) -> UpsertMessage {
    UpsertMessage::Artist(Box::new(artist::ActiveModel {
        id: Set(id.to_string()),
        name: Set(id.to_string()),
        artist_image_url: Set(None),
        user_rating: Set(0),
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
        user_rating: Set(0),
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
        user_rating: Set(0),
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
        user_rating: Set(0),
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
        user_rating: Set(0),
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
            user_rating: Set(0),
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
            user_rating: Set(0),
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
