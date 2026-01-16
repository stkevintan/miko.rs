use serde::Deserialize;
use std::env;
use dotenvy::dotenv;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub subsonic: SubsonicConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub jwt_secret: String,
    pub password_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubsonicConfig {
    pub data_dir: String,
    pub ignored_articles: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok();

        let read_val = |key: &str, default: Option<&str>| -> String {
            env::var(key).unwrap_or_else(|_| default.unwrap_or_default().to_string())
        };

        let read_path = |key: &str, default: Option<&str>| -> String {
            norm_path(&read_val(key, default))
        };

        Ok(Config {
            server: ServerConfig {
                port: read_val("PORT", Some("8081")).parse()?,
                jwt_secret: read_val("JWT_SECRET", None),
                password_secret: read_val("PASSWORD_SECRET", None),
            },
            database: DatabaseConfig {
                url: norm_path(&read_val("DATABASE_URL", None)),
            },
            subsonic: SubsonicConfig {
                data_dir: read_path("SUBSONIC_DATA_DIR", Some("./data")),
                ignored_articles: read_val("SUBSONIC_IGNORED_ARTICLES", Some("The El La Los Las Le Les")),
            },
        })
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.server.jwt_secret.is_empty() {
            anyhow::bail!("JWT_SECRET is required");
        }
        if self.server.password_secret.is_empty() {
            anyhow::bail!("PASSWORD_SECRET is required");
        }
        if self.database.url.is_empty() {
            anyhow::bail!("DATABASE_URL is required");
        }
        Ok(())
    }
}

fn norm_path(path: &str) -> String {
    expand_path(path).replace('\\', "/")
}

fn expand_path(path: &str) -> String {
    let home = env::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    path.replacen('~', &home, 1).replace("$HOME", &home)
}
