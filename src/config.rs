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
    pub folders: Vec<String>,
    pub ignored_articles: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok();

        let read_val = |key: &str, default: Option<&str>| -> String {
            env::var(key).unwrap_or_else(|_| default.unwrap_or_default().to_string())
        };

        let read_path = |key: &str, default: Option<&str>| -> String {
            expand_path(&read_val(key, default))
        };

        Ok(Config {
            server: ServerConfig {
                port: read_val("PORT", Some("8081")).parse()?,
                jwt_secret: read_val("JWT_SECRET", None),
                password_secret: read_val("PASSWORD_SECRET", None),
            },
            database: DatabaseConfig {
                url: expand_path(&read_val("DATABASE_URL", None)),
            },
            subsonic: SubsonicConfig {
                data_dir: read_path("SUBSONIC_DATA_DIR", Some("./data")),
                folders: read_val("SUBSONIC_MUSIC_FOLDERS", None)
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(expand_path)
                    .collect(),
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

fn expand_path(path: &str) -> String {
    if '~' != path.chars().next().unwrap_or(' ') && !path.contains("$HOME") {
        return path.to_string();
    }
    let home = env::var("HOME").expect("HOME environment variable not set");
    path.replacen('~', &home, 1).replace("$HOME", &home)
}
