use actix_web::{web, HttpResponse};
use crate::subsonic::models::SubsonicResponse;
use serde::{Deserialize, Serialize};

pub mod system;
pub mod browsing;

macro_rules! subsonic_routes {
    ($scope:expr, $(($path:literal, $handler:expr)),* $(,)?) => {
        $scope
            $(
                .route($path, web::get().to($handler))
                .route(concat!($path, ".view"), web::get().to($handler))
            )*
    };
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SubsonicParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub v: Option<String>,
    pub c: Option<String>,
    pub f: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        subsonic_routes!(
            web::scope("/rest")
                .wrap(crate::subsonic::middleware::SubsonicAuth),
            // system
            ("/ping", system::ping),
            ("/getLicense", system::get_license),
            ("/getOpenSubsonicExtensions", system::get_open_subsonic_extensions),
            // browsing
            ("/getMusicFolders", browsing::get_music_folders),
            ("/getIndexes", browsing::get_indexes),
            ("/getMusicDirectory", browsing::get_music_directory),
            ("/getGenres", browsing::get_genres),
            ("/getArtists", browsing::get_artists),
            ("/getArtist", browsing::get_artist),
            ("/getAlbum", browsing::get_album),
            ("/getSong", browsing::get_song),
        )
    );
}

pub fn send_response<T: Serialize>(resp: SubsonicResponse<T>, format: &Option<String>) -> HttpResponse {
    let is_json = format.as_deref() == Some("json");
    
    if is_json {
        let mut val = serde_json::to_value(&resp).unwrap_or_default();
        clean_json_attributes(&mut val);
        HttpResponse::Ok()
            .content_type("application/json")
            .json(serde_json::json!({ "subsonic-response": val }))
    } else {
        let xml = quick_xml::se::to_string(&resp).unwrap_or_default();
        let xml_header = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
        HttpResponse::Ok()
            .content_type("application/xml")
            .body(format!("{}{}", xml_header, xml))
    }
}

fn clean_json_attributes(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let old_map = std::mem::take(map);
            for (k, mut v) in old_map {
                clean_json_attributes(&mut v);
                let new_key = if k.starts_with(&['@', '$']) {
                    k[1..].to_string()
                } else {
                    k
                };
                map.insert(new_key, v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                clean_json_attributes(v);
            }
        }
        _ => {}
    }
}
