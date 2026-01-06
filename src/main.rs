mod models;
mod subsonic;
mod crypto;
mod api;

use poem::{Server, listener::TcpListener, Route, middleware::Tracing, EndpointExt};
use sea_orm::{Database, EntityTrait, ActiveModelTrait, Set, DatabaseConnection, PaginatorTrait, Schema, ConnectionTrait};
use std::env;
use dotenvy::dotenv;
use chrono::Utc;
use crate::models::{user, music_folder, user_music_folder, child, artist, album, genre};

async fn setup_schema(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    // Create tables if they don't exist
    let tables = [
        schema.create_table_from_entity(user::Entity),
        schema.create_table_from_entity(music_folder::Entity),
        schema.create_table_from_entity(user_music_folder::Entity),
        schema.create_table_from_entity(child::Entity),
        schema.create_table_from_entity(artist::Entity),
        schema.create_table_from_entity(album::Entity),
        schema.create_table_from_entity(genre::Entity),
    ];

    for mut table in tables {
        let stmt = builder.build(table.if_not_exists());
        db.execute(stmt).await?;
    }

    Ok(())
}

async fn init_default_user(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    let count = user::Entity::find().count(db).await?;
    if count == 0 {
        log::info!("No users found, creating default admin user");
        let secret = env::var("PASSWORD_SECRET").unwrap_or_default();
        let encrypted_password = crypto::encrypt("adminpassword", secret.as_bytes())?;
        
        let admin = user::ActiveModel {
            username: Set("admin".to_string()),
            password: Set(encrypted_password),
            email: Set(Some("admin@example.com".to_string())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            scrobbling_enabled: Set(true),
            settings_role: Set(true),
            download_role: Set(true),
            upload_role: Set(true),
            admin_role: Set(true),
            playlist_role: Set(true),
            cover_art_role: Set(true),
            comment_role: Set(true),
            podcast_role: Set(true),
            stream_role: Set(true),
            jukebox_role: Set(true),
            share_role: Set(true),
            video_conversion_role: Set(true),
            ..Default::default()
        };
        
        admin.insert(db).await?;
        log::info!("Default admin user created");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    setup_schema(&db).await.expect("Failed to setup database schema");
    init_default_user(&db).await.expect("Failed to initialize default user");

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    log::info!("Starting server at http://{}", addr);

    let app = Route::new()
        .nest("/rest", subsonic::create_route())
        .nest("/api", api::create_route())
        .data(db)
        .with(Tracing);

    Server::new(TcpListener::bind(addr))
        .run(app)
        .await?;
        
    Ok(())
}
