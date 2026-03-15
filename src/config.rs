use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

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

        let read_path =
            |key: &str, default: Option<&str>| -> String { norm_path(&read_val(key, default)) };

        Ok(Config {
            server: ServerConfig {
                port: read_val("PORT", Some("8081")).parse()?,
                jwt_secret: read_val("JWT_SECRET", None),
                password_secret: read_val("PASSWORD_SECRET", None),
            },
            database: DatabaseConfig {
                url: normalize_database_url(&read_val("DATABASE_URL", None)),
            },
            subsonic: SubsonicConfig {
                data_dir: read_path("SUBSONIC_DATA_DIR", Some("./data")),
                ignored_articles: read_val(
                    "SUBSONIC_IGNORED_ARTICLES",
                    Some("The El La Los Las Le Les"),
                ),
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

/// Normalize a sqlite:// URL so that $HOME/~ expansion produces a valid path
/// on both Unix (sqlite:///absolute/path) and Windows (sqlite://C:/path).
fn normalize_database_url(url: &str) -> String {
    let rest = if let Some(rest) = url.strip_prefix("sqlite://") {
        rest.trim_start_matches('/')
    } else if let Some(rest) = url.strip_prefix("sqlite:") {
        rest
    } else {
        return norm_path(url);
    };

    let expanded = expand_path(rest).replace('\\', "/");
    format!("sqlite://{}", expanded)
}

fn norm_path(path: &str) -> String {
    expand_path(path).replace('\\', "/")
}

fn expand_path(path: &str) -> String {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let path = if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
        format!("{}{}", home, &path[1..])
    } else {
        path.to_string()
    };
    path.replace("$HOME", &home)
}
