use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Ok,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponse {
    #[serde(rename = "@status")]
    pub status: ResponseStatus,
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(rename = "@serverVersion", skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
    #[serde(rename = "@openSubsonic", skip_serializing_if = "Option::is_none")]
    pub open_subsonic: Option<bool>,
    
    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicError>,
    
    #[serde(rename = "ping", skip_serializing_if = "Option::is_none")]
    pub ping: Option<()>,

    #[serde(rename = "license", skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,

    #[serde(rename = "musicFolders", skip_serializing_if = "Option::is_none")]
    pub music_folders: Option<MusicFolders>,

    #[serde(rename = "openSubsonicExtensions", skip_serializing_if = "Option::is_none")]
    pub open_subsonic_extensions: Option<OpenSubsonicExtensions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSubsonicExtensions {
    #[serde(rename = "extension")]
    pub extension: Vec<OpenSubsonicExtension>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSubsonicExtension {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@versions")]
    pub versions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicError {
    #[serde(rename = "@code")]
    pub code: i32,
    #[serde(rename = "@message")]
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    #[serde(rename = "@valid")]
    pub valid: bool,
    #[serde(rename = "@email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "@licenseExpires", skip_serializing_if = "Option::is_none")]
    pub license_expires: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "@trialExpires", skip_serializing_if = "Option::is_none")]
    pub trial_expires: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFolders {
    #[serde(rename = "musicFolder")]
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicFolder {
    #[serde(rename = "@id")]
    pub id: i32,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl SubsonicResponse {
    pub fn new_ok(version: String) -> Self {
        Self {
            status: ResponseStatus::Ok,
            version,
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            error: None,
            ping: None,
            license: None,
            music_folders: None,
            open_subsonic_extensions: None,
        }
    }

    pub fn new_error(code: i32, message: String, version: String) -> Self {
        Self {
            status: ResponseStatus::Failed,
            version,
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            server_version: Some("1.0.0".to_string()),
            open_subsonic: Some(true),
            error: Some(SubsonicError { code, message }),
            ping: None,
            license: None,
            music_folders: None,
            open_subsonic_extensions: None,
        }
    }
}
