use super::*;
use crate::models::{user, user_rating, user_star};
use chrono::Utc;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    // Create a test user (required by FK constraints)
    let now = Utc::now();
    user::ActiveModel {
        username: Set("testuser".to_string()),
        password: Set("pass".to_string()),
        email: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        scrobbling_enabled: Set(true),
        settings_role: Set(false),
        download_role: Set(true),
        upload_role: Set(false),
        admin_role: Set(false),
        playlist_role: Set(true),
        cover_art_role: Set(true),
        comment_role: Set(true),
        podcast_role: Set(false),
        stream_role: Set(true),
        jukebox_role: Set(false),
        share_role: Set(true),
        video_conversion_role: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    db
}

// ─── insert_stars ──────────────────────────────────────────────

#[tokio::test]
async fn insert_stars_song() {
    let db = setup_db().await;

    let query = StarQuery {
        id: vec!["song1".into(), "song2".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "testuser", query).await.unwrap();

    let stars = user_star::Entity::find().all(&db).await.unwrap();
    assert_eq!(stars.len(), 2);
    assert!(stars.iter().all(|s| s.item_type == "song"));
    assert!(stars.iter().all(|s| s.username == "testuser"));
}

#[tokio::test]
async fn insert_stars_album_and_artist() {
    let db = setup_db().await;

    let query = StarQuery {
        id: vec![],
        album_id: vec!["album1".into()],
        artist_id: vec!["artist1".into()],
    };
    insert_stars(&db, "testuser", query).await.unwrap();

    let stars = user_star::Entity::find().all(&db).await.unwrap();
    assert_eq!(stars.len(), 2);

    let album_star = stars.iter().find(|s| s.item_type == "album").unwrap();
    assert_eq!(album_star.item_id, "album1");

    let artist_star = stars.iter().find(|s| s.item_type == "artist").unwrap();
    assert_eq!(artist_star.item_id, "artist1");
}

#[tokio::test]
async fn insert_stars_duplicate_is_idempotent() {
    let db = setup_db().await;

    let query1 = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "testuser", query1).await.unwrap();

    // Insert the same star again — should not fail or create duplicates
    let query2 = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "testuser", query2).await.unwrap();

    let stars = user_star::Entity::find().all(&db).await.unwrap();
    assert_eq!(stars.len(), 1);
}

// ─── remove_stars ──────────────────────────────────────────────

#[tokio::test]
async fn remove_stars_deletes_matching() {
    let db = setup_db().await;

    // Insert stars for songs, albums, and artists
    let query = StarQuery {
        id: vec!["song1".into(), "song2".into()],
        album_id: vec!["album1".into()],
        artist_id: vec!["artist1".into()],
    };
    insert_stars(&db, "testuser", query).await.unwrap();
    assert_eq!(user_star::Entity::find().all(&db).await.unwrap().len(), 4);

    // Remove only song1
    let remove_query = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    remove_stars(&db, "testuser", remove_query).await.unwrap();

    let remaining = user_star::Entity::find().all(&db).await.unwrap();
    assert_eq!(remaining.len(), 3);
    assert!(!remaining
        .iter()
        .any(|s| s.item_id == "song1" && s.item_type == "song"));
}

#[tokio::test]
async fn remove_stars_does_not_affect_other_users() {
    let db = setup_db().await;

    // Create a second user
    let now = Utc::now();
    user::ActiveModel {
        username: Set("otheruser".to_string()),
        password: Set("pass".to_string()),
        email: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        scrobbling_enabled: Set(true),
        settings_role: Set(false),
        download_role: Set(true),
        upload_role: Set(false),
        admin_role: Set(false),
        playlist_role: Set(true),
        cover_art_role: Set(true),
        comment_role: Set(true),
        podcast_role: Set(false),
        stream_role: Set(true),
        jukebox_role: Set(false),
        share_role: Set(true),
        video_conversion_role: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Both users star the same song
    let q1 = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "testuser", q1).await.unwrap();
    let q2 = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "otheruser", q2).await.unwrap();

    // Remove testuser's star
    let rm = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    remove_stars(&db, "testuser", rm).await.unwrap();

    let remaining = user_star::Entity::find().all(&db).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].username, "otheruser");
}

#[tokio::test]
async fn remove_stars_empty_query_is_noop() {
    let db = setup_db().await;

    let q = StarQuery {
        id: vec!["song1".into()],
        album_id: vec![],
        artist_id: vec![],
    };
    insert_stars(&db, "testuser", q).await.unwrap();

    // Remove with empty lists — should not delete anything
    let empty = StarQuery {
        id: vec![],
        album_id: vec![],
        artist_id: vec![],
    };
    remove_stars(&db, "testuser", empty).await.unwrap();

    assert_eq!(user_star::Entity::find().all(&db).await.unwrap().len(), 1);
}

// ─── user_rating upsert / delete ───────────────────────────────

#[tokio::test]
async fn upsert_rating_creates_new() {
    let db = setup_db().await;

    let rating = user_rating::ActiveModel {
        username: Set("testuser".to_string()),
        item_id: Set("song1".to_string()),
        item_type: Set("song".to_string()),
        rating: Set(4),
    };
    user_rating::Entity::insert(rating)
        .exec_without_returning(&db)
        .await
        .unwrap();

    let ratings = user_rating::Entity::find().all(&db).await.unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0].rating, 4);
}

#[tokio::test]
async fn upsert_rating_updates_existing() {
    let db = setup_db().await;

    // Insert initial rating
    let rating1 = user_rating::ActiveModel {
        username: Set("testuser".to_string()),
        item_id: Set("song1".to_string()),
        item_type: Set("song".to_string()),
        rating: Set(3),
    };
    user_rating::Entity::insert(rating1)
        .exec_without_returning(&db)
        .await
        .unwrap();

    // Upsert with new value
    let rating2 = user_rating::ActiveModel {
        username: Set("testuser".to_string()),
        item_id: Set("song1".to_string()),
        item_type: Set("song".to_string()),
        rating: Set(5),
    };
    user_rating::Entity::insert(rating2)
        .on_conflict(
            sea_orm::sea_query::OnConflict::columns([
                user_rating::Column::Username,
                user_rating::Column::ItemId,
                user_rating::Column::ItemType,
            ])
            .update_column(user_rating::Column::Rating)
            .to_owned(),
        )
        .exec_without_returning(&db)
        .await
        .unwrap();

    let ratings = user_rating::Entity::find().all(&db).await.unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0].rating, 5);
}

#[tokio::test]
async fn delete_rating_removes_entry() {
    let db = setup_db().await;

    let rating = user_rating::ActiveModel {
        username: Set("testuser".to_string()),
        item_id: Set("song1".to_string()),
        item_type: Set("song".to_string()),
        rating: Set(4),
    };
    user_rating::Entity::insert(rating)
        .exec_without_returning(&db)
        .await
        .unwrap();

    // Delete rating (simulates set_rating with rating=0)
    user_rating::Entity::delete_many()
        .filter(user_rating::Column::Username.eq("testuser"))
        .filter(user_rating::Column::ItemId.eq("song1"))
        .filter(user_rating::Column::ItemType.eq("song"))
        .exec(&db)
        .await
        .unwrap();

    let ratings = user_rating::Entity::find().all(&db).await.unwrap();
    assert_eq!(ratings.len(), 0);
}

#[tokio::test]
async fn rating_does_not_affect_other_users() {
    let db = setup_db().await;

    // Create second user
    let now = Utc::now();
    user::ActiveModel {
        username: Set("otheruser".to_string()),
        password: Set("pass".to_string()),
        email: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        scrobbling_enabled: Set(true),
        settings_role: Set(false),
        download_role: Set(true),
        upload_role: Set(false),
        admin_role: Set(false),
        playlist_role: Set(true),
        cover_art_role: Set(true),
        comment_role: Set(true),
        podcast_role: Set(false),
        stream_role: Set(true),
        jukebox_role: Set(false),
        share_role: Set(true),
        video_conversion_role: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Both users rate the same song
    for (user, rating_val) in [("testuser", 3), ("otheruser", 5)] {
        let r = user_rating::ActiveModel {
            username: Set(user.to_string()),
            item_id: Set("song1".to_string()),
            item_type: Set("song".to_string()),
            rating: Set(rating_val),
        };
        user_rating::Entity::insert(r)
            .exec_without_returning(&db)
            .await
            .unwrap();
    }

    // Delete testuser's rating
    user_rating::Entity::delete_many()
        .filter(user_rating::Column::Username.eq("testuser"))
        .filter(user_rating::Column::ItemId.eq("song1"))
        .exec(&db)
        .await
        .unwrap();

    let ratings = user_rating::Entity::find().all(&db).await.unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0].username, "otheruser");
    assert_eq!(ratings[0].rating, 5);
}
