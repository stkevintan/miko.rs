mod models;
mod subsonic;
mod crypto;
mod api;
mod config;
mod scanner;

use poem::{Server, listener::TcpListener, Route, middleware::Tracing, EndpointExt};
use sea_orm::{Database, EntityTrait, ActiveModelTrait, Set, DatabaseConnection, PaginatorTrait, Schema, ConnectionTrait, QueryFilter, ColumnTrait};
use chrono::Utc;
use std::sync::Arc;
use crate::models::{user, music_folder, user_music_folder, child, artist, album, genre, song_artist, song_genre};
use crate::config::Config;
use crate::scanner::Scanner;

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
        schema.create_table_from_entity(song_artist::Entity),
        schema.create_table_from_entity(song_genre::Entity),
    ];

    for mut table in tables {
        let stmt = builder.build(table.if_not_exists());
        db.execute(stmt).await?;
    }

    Ok(())
}

async fn init_default_user(db: &DatabaseConnection, password_secret: &str) -> Result<(), anyhow::Error> {
    let count = user::Entity::find().count(db).await?;
    if count == 0 {
        log::info!("No users found, creating default admin user");
        let encrypted_password = crypto::encrypt("adminpassword", password_secret.as_bytes())?;
        
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

async fn init_music_folders(db: &DatabaseConnection, folders: &[String]) -> Result<(), anyhow::Error> {
    for path in folders {
        let exists = music_folder::Entity::find()
            .filter(music_folder::Column::Path.eq(path))
            .one(db)
            .await?;

        if exists.is_none() {
            log::info!("Adding music folder from config: {}", path);
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Music".to_string());

            let folder = music_folder::ActiveModel {
                path: Set(path.clone()),
                name: Set(Some(name)),
                ..Default::default()
            };
            folder.insert(db).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::load()?;
    let config = Arc::new(config);
    config.validate()?;

    // Ensure database directory exists for SQLite
    if config.database.url.starts_with("sqlite://") {
        let path_part = config.database.url
            .strip_prefix("sqlite://").unwrap()
            .split('?').next().unwrap();
        let path = std::path::Path::new(path_part);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                log::info!("Creating database directory: {:?}", parent);
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let mut opt = sea_orm::ConnectOptions::new(&config.database.url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(std::time::Duration::from_secs(30))
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .sqlx_logging(false);

    let db = Database::connect(opt)
        .await
        .expect("Failed to connect to database");

    setup_schema(&db).await.expect("Failed to setup database schema");
    init_default_user(&db, &config.server.password_secret).await.expect("Failed to initialize default user");
    init_music_folders(&db, &config.subsonic.folders).await.expect("Failed to initialize music folders");

    let scanner = Arc::new(Scanner::new(db.clone(), config.clone()));
    let addr = format!("0.0.0.0:{}", config.server.port);

    log::info!("Starting server at http://{}", addr);

    let app = Route::new()
        .nest("/rest", subsonic::create_route())
        .nest("/api", api::create_route())
        .data(db)
        .data(config)
        .data(scanner)
        .with(Tracing);

    Server::new(TcpListener::bind(addr))
        .run(app)
        .await?;
        
    Ok(())
}
