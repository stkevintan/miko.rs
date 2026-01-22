use serde::Deserialize;
use anyhow::{Result, anyhow};
use log::{debug, warn};
use std::time::Duration;
use governor::{Quota, RateLimiter, DefaultDirectRateLimiter};
use std::num::NonZeroU32;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Deserialize)]
pub struct MBArtist {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBArtistCredit {
    pub name: String,
    pub artist: MBArtist,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBGenre {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBLabel {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBLabelInfo {
    #[serde(rename = "catalog-number")]
    pub catalog_number: Option<String>,
    pub label: Option<MBLabel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBReleaseGroup {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBTrack {
    pub id: String,
    pub position: Option<u32>,
    pub recording: Option<MBRecordingMinimal>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBRecordingMinimal {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBMedium {
    pub position: Option<u32>,
    pub tracks: Option<Vec<MBTrack>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBCoverArtArchive {
    pub front: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBRelease {
    pub id: String,
    pub title: String,
    pub date: Option<String>,
    pub barcode: Option<String>,
    pub asin: Option<String>,
    #[serde(rename = "release-group")]
    pub release_group: Option<MBReleaseGroup>,
    #[serde(rename = "label-info")]
    pub label_info: Option<Vec<MBLabelInfo>>,
    pub media: Option<Vec<MBMedium>>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<MBArtistCredit>>,
    #[serde(rename = "cover-art-archive")]
    pub cover_art_archive: Option<MBCoverArtArchive>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MBRecording {
    pub id: String,
    pub title: String,
    pub isrcs: Option<Vec<String>>,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Option<Vec<MBArtistCredit>>,
    pub genres: Option<Vec<MBGenre>>,
    pub releases: Option<Vec<MBRelease>>,
}

#[derive(Debug, Deserialize)]
pub struct MBSearchResponse {
    pub recordings: Vec<MBRecording>,
}

pub struct MusicBrainzClient {
    client: reqwest::Client,
    user_agent: String,
    rate_limiter: DefaultDirectRateLimiter,
}

impl MusicBrainzClient {
    pub fn new(app_name: &str, version: &str, contact: &str) -> Result<Self> {
        let user_agent = format!("{}/{}/{} ( {} )", app_name, version, app_name, contact);
        
        // MusicBrainz allows 1 request per second
        let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
        let rate_limiter = RateLimiter::direct(quota);

        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?,
            user_agent,
            rate_limiter,
        })
    }

    async fn request_with_retry<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = anyhow!("Unknown error");

        while attempts < max_attempts {
            attempts += 1;
            
            // Wait for rate limiter
            self.rate_limiter.until_ready().await;

            debug!("MusicBrainz Request (Attempt {}/{}): {}", attempts, max_attempts, url);

            match self.client.get(url)
                .header("User-Agent", &self.user_agent)
                .send()
                .await 
            {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(response.json().await?);
                    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                        warn!("MusicBrainz error {}: {}. Retrying in {}s...", status, url, attempts * 2);
                        // Exponential backoff
                        tokio::time::sleep(Duration::from_secs(attempts * 2)).await;
                        last_error = anyhow!("MusicBrainz returned status: {}", status);
                        continue;
                    } else {
                        return Err(anyhow!("MusicBrainz request failed with status: {}", status));
                    }
                }
                Err(e) => {
                    warn!("Network error connecting to MusicBrainz: {}. Retrying in {}s...", e, attempts * 2);
                    tokio::time::sleep(Duration::from_secs(attempts * 2)).await;
                    last_error = anyhow::Error::from(e);
                    continue;
                }
            }
        }

        Err(last_error)
    }

    pub async fn search_recording(&self, lucene_query: &str) -> Result<Vec<MBRecording>> {
        let url = format!("https://musicbrainz.org/ws/2/recording?query={}&fmt=json", 
            urlencoding::encode(lucene_query));
        
        let result: MBSearchResponse = self.request_with_retry(&url).await?;
        Ok(result.recordings)
    }

    pub async fn fetch_recording(&self, mbid: &str) -> Result<MBRecording> {
        let url = format!("https://musicbrainz.org/ws/2/recording/{}?inc=artist-credits+releases+genres+isrcs+media&fmt=json", 
            urlencoding::encode(mbid));
        
        self.request_with_retry(&url).await
    }

    pub async fn fetch_cover_art(&self, release_mbid: &str) -> Result<Option<String>> {
        let url = format!("https://coverartarchive.org/release/{}/front", 
            urlencoding::encode(release_mbid));
        
        debug!("Fetching cover art from CAA for release: {}", release_mbid);
        
        // CAA is not rate-limited by the same 1rps rules as MB, but we still use the client
        let response = self.client.get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("CAA request failed with status: {}", response.status()));
        }

        let mime_type = response.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let bytes = response.bytes().await?;
        let base64_image = general_purpose::STANDARD.encode(bytes);
        
        Ok(Some(format!("data:{};base64,{}", mime_type, base64_image)))
    }
}
