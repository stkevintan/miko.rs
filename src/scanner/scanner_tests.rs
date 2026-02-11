use super::*;
use crate::scanner::types::UpsertMessage;
use sea_orm::Database;

/// Create a Scanner backed by an in-memory SQLite DB (no migrations needed for unit tests).
async fn test_scanner() -> Scanner {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let cfg = test_config();
    Scanner::new(db, cfg)
}

fn test_config() -> Arc<Config> {
    Arc::new(Config {
        server: crate::config::ServerConfig {
            port: 8081,
            jwt_secret: "test".to_string(),
            password_secret: "test".to_string(),
        },
        database: crate::config::DatabaseConfig {
            url: "sqlite::memory:".to_string(),
        },
        subsonic: crate::config::SubsonicConfig {
            data_dir: "/tmp/miko-test".to_string(),
            ignored_articles: "The".to_string(),
        },
    })
}

// ─── build_artist ────────────────────────────────────────────────

#[tokio::test]
async fn build_artist_pushes_artist_message() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let id = scanner.build_artist_test("Radiohead", &mut batch);

    assert_eq!(id, utils::generate_artist_id("Radiohead"));
    assert_eq!(batch.len(), 1);
    match &batch[0] {
        UpsertMessage::Artist(a) => {
            assert_eq!(*a.id.as_ref(), id);
            assert_eq!(*a.name.as_ref(), "Radiohead");
        }
        _ => panic!("Expected Artist message"),
    }
}

#[tokio::test]
async fn build_artist_deterministic_id() {
    let scanner = test_scanner().await;
    let mut b1 = Vec::new();
    let mut b2 = Vec::new();
    let id1 = scanner.build_artist_test("Radiohead", &mut b1);
    let id2 = scanner.build_artist_test("Radiohead", &mut b2);
    assert_eq!(id1, id2);
}

// ─── build_genre ─────────────────────────────────────────────────

#[tokio::test]
async fn build_genre_pushes_genre_message() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let name = scanner.build_genre_test("Rock", &mut batch);

    assert_eq!(name, "Rock");
    assert_eq!(batch.len(), 1);
    match &batch[0] {
        UpsertMessage::Genre(g) => {
            assert_eq!(*g.name.as_ref(), "Rock");
        }
        _ => panic!("Expected Genre message"),
    }
}

#[tokio::test]
async fn build_genre_trims_whitespace() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let name = scanner.build_genre_test("  Jazz  ", &mut batch);
    assert_eq!(name, "Jazz");
    match &batch[0] {
        UpsertMessage::Genre(g) => assert_eq!(*g.name.as_ref(), "Jazz"),
        _ => panic!("Expected Genre message"),
    }
}

// ─── build_album ─────────────────────────────────────────────────

#[tokio::test]
async fn build_album_pushes_correct_messages() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let created = chrono::Utc::now();
    let album_id = scanner.build_album_test(
        "OK Computer",
        &["Radiohead"],
        1997,
        &["Alternative Rock"],
        created,
        &mut batch,
    );

    let expected_id = utils::generate_album_id("Radiohead", "OK Computer");
    assert_eq!(album_id, expected_id);

    // Expected messages in order:
    //   Album, Artist (for album_artist), Genre (for album_genre), AlbumRelations
    assert_eq!(batch.len(), 4);
    assert!(matches!(&batch[0], UpsertMessage::Album(_)));
    assert!(matches!(&batch[1], UpsertMessage::Artist(_)));
    assert!(matches!(&batch[2], UpsertMessage::Genre(_)));
    assert!(matches!(&batch[3], UpsertMessage::AlbumRelations(_)));
}

#[tokio::test]
async fn build_album_with_multiple_artists() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let created = chrono::Utc::now();
    let album_id = scanner.build_album_test(
        "Collab",
        &["Artist A", "Artist B"],
        2020,
        &[],
        created,
        &mut batch,
    );

    let expected_id = utils::generate_album_id("Artist A; Artist B", "Collab");
    assert_eq!(album_id, expected_id);

    // Album + 2 Artists + AlbumRelations = 4
    assert_eq!(batch.len(), 4);

    // Check that AlbumRelations contains both artists
    match &batch[3] {
        UpsertMessage::AlbumRelations(r) => {
            assert_eq!(r.artists.len(), 2);
            assert_eq!(r.album_id, expected_id);
        }
        _ => panic!("Expected AlbumRelations"),
    }
}

#[tokio::test]
async fn build_album_skips_empty_genres() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let created = chrono::Utc::now();
    scanner.build_album_test(
        "Album",
        &["Artist"],
        2020,
        &["Rock", "", "  ", "Jazz"],
        created,
        &mut batch,
    );

    // Album + 1 Artist + 2 Genres (Rock, Jazz) + AlbumRelations = 5
    assert_eq!(batch.len(), 5);

    match &batch[4] {
        UpsertMessage::AlbumRelations(r) => {
            assert_eq!(r.genres.len(), 2);
            assert!(r.genres.contains(&"Rock".to_string()));
            assert!(r.genres.contains(&"Jazz".to_string()));
        }
        _ => panic!("Expected AlbumRelations"),
    }
}

// ─── message ordering in batch ───────────────────────────────────

#[tokio::test]
async fn build_album_dependencies_come_before_relations() {
    let scanner = test_scanner().await;
    let mut batch = Vec::new();
    let created = chrono::Utc::now();
    scanner.build_album_test(
        "Album X",
        &["ArtistX"],
        2020,
        &["GenreX"],
        created,
        &mut batch,
    );

    // The Album message must appear before AlbumRelations
    let album_pos = batch.iter().position(|m| matches!(m, UpsertMessage::Album(_))).unwrap();
    let rel_pos = batch.iter().position(|m| matches!(m, UpsertMessage::AlbumRelations(_))).unwrap();
    assert!(album_pos < rel_pos, "Album must be before AlbumRelations");

    // Artist must appear before AlbumRelations
    let artist_pos = batch.iter().position(|m| matches!(m, UpsertMessage::Artist(_))).unwrap();
    assert!(artist_pos < rel_pos, "Artist must be before AlbumRelations");

    // Genre must appear before AlbumRelations
    let genre_pos = batch.iter().position(|m| matches!(m, UpsertMessage::Genre(_))).unwrap();
    assert!(genre_pos < rel_pos, "Genre must be before AlbumRelations");
}
