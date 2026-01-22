use serde::Deserialize;
use anyhow::{Result, anyhow};
use log::debug;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct LrcLibLyrics {
    pub instrumental: bool,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

pub struct LyricsService {
    client: reqwest::Client,
    user_agent: String,
}

impl LyricsService {
    pub fn new() -> Result<Self> {
        let user_agent = format!("{}/{} ( {} )", 
            env!("CARGO_PKG_NAME"), 
            env!("CARGO_PKG_VERSION"), 
            env!("CARGO_PKG_REPOSITORY"));
        
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?,
            user_agent,
        })
    }

    pub async fn fetch_lyrics(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: u32,
    ) -> Result<Option<String>> {
        let mut url = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}&duration={}",
            urlencoding::encode(artist),
            urlencoding::encode(title),
            duration_secs
        );

        if let Some(alb) = album {
            url.push_str(&format!("&album_name={}", urlencoding::encode(alb)));
        }

        debug!("Fetching lyrics from LRCLIB: {}", url);

        let response = self.client.get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            debug!("Lyrics not found for: {} by {}", title, artist);
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("LRCLIB request failed with status: {}", response.status()));
        }

        let lyrics_data: LrcLibLyrics = response.json().await?;
        
        if lyrics_data.instrumental {
            return Ok(Some("[Instrumental]".to_string()));
        }

        // Prefer synced lyrics if available, otherwise plain
        let lyrics = lyrics_data.synced_lyrics
            .or(lyrics_data.plain_lyrics);

        Ok(lyrics)
    }
}
