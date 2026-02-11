use crate::config::Config;
use md5;
use std::path::{Path, PathBuf};

pub fn get_cover_cache_dir(cfg: &Config) -> PathBuf {
    Path::new(&cfg.subsonic.data_dir)
        .join("cache")
        .join("covers")
}

pub fn generate_id(path: &str, folder_id: i32, folder_path: &str) -> String {
    let rel = match pathdiff::diff_paths(path, folder_path) {
        Some(p) => p.to_string_lossy().replace('\\', "/"),
        None => path.replace('\\', "/"),
    };
    let data = format!("{}:{}", folder_id, rel);
    format!("{:x}", md5::compute(data))
}

pub fn generate_album_id(artist: &str, album: &str) -> String {
    let data = format!("{}|{}", artist, album);
    format!("{:x}", md5::compute(data))
}

pub fn generate_artist_id(name: &str) -> String {
    format!("{:x}", md5::compute(name))
}

pub fn get_parent_id(path: &str, folder_id: i32, folder_path: &str) -> Option<String> {
    if path == folder_path {
        return None;
    }

    let path_obj = Path::new(path);
    let parent = path_obj.parent()?;
    let parent_str = parent.to_string_lossy().replace('\\', "/");

    // Ensure we don't go above folder_path
    if !parent_str.starts_with(folder_path) && parent_str != folder_path {
        return None;
    }

    Some(generate_id(&parent_str, folder_id, folder_path))
}

pub fn is_audio_file(ext: &str) -> bool {
    matches!(ext, "mp3" | "flac" | "m4a" | "wav" | "ogg" | "opus")
}
