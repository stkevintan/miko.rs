use poem::Response;
use crate::subsonic::models::SubsonicResponse;
use serde::{Deserialize, Deserializer};

pub fn deserialize_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        One(T),
        Many(Vec<T>),
    }

    match OneOrMany::<T>::deserialize(deserializer)? {
        OneOrMany::One(x) => Ok(vec![x]),
        OneOrMany::Many(xs) => Ok(xs),
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct SubsonicParams {
    pub u: Option<String>,
    pub p: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub c: Option<String>,
    pub f: Option<String>,
}

pub fn send_response(resp: SubsonicResponse, format: &Option<String>) -> Response {
    let is_json = format.as_deref() == Some("json");
    
    if is_json {
        match serde_json::to_value(&resp) {
            Ok(mut val) => {
                clean_json_attributes(&mut val);
                Response::builder()
                    .header("content-type", "application/json")
                    .body(serde_json::to_string(&serde_json::json!({ "subsonic-response": val })).unwrap())
            }
            Err(e) => {
                log::error!("Failed to serialize SubsonicResponse to JSON: {}", e);
                Response::builder()
                    .status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .finish()
            }
        }
    } else {
        match quick_xml::se::to_string(&resp) {
            Ok(xml) => {
                let xml_header = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
                Response::builder()
                    .header("content-type", "application/xml")
                    .body(format!("{}{}", xml_header, xml))
            }
            Err(e) => {
                log::error!("Failed to serialize SubsonicResponse to XML: {}", e);
                Response::builder()
                    .status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
                    .finish()
            }
        }
    }
}

pub fn clean_json_attributes(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // Remove xmlns/status from JSON for Subsonic compatibility if needed
            map.remove("@xmlns");

            // If there's a "$value" key, we want to handle it
            if let Some(v) = map.remove("$value") {
                match v {
                    serde_json::Value::Object(inner_map) => {
                        // Flatten contents if it's an object
                        for (k, v) in inner_map {
                            map.insert(k, v);
                        }
                    }
                    _ => {
                        // Otherwise rename to "value" for Subsonic JSON compatibility
                        map.insert("value".to_string(), v);
                    }
                }
            }

            let old_map = std::mem::take(map);
            for (k, mut v) in old_map {
                clean_json_attributes(&mut v);
                let new_key = if k.starts_with('@') {
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
