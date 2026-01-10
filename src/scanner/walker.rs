use walkdir::WalkDir;
use crate::{models::music_folder, scanner::utils::is_audio_file};
use tokio::sync::mpsc;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WalkTask {
    pub path: String,
    pub is_dir: bool,
    pub name: String,
    pub ext: String,
    pub size: u64,
    pub mod_time: chrono::DateTime<chrono::Utc>,
    pub folder: music_folder::Model,
}

pub struct Walker;

impl Walker {
    pub fn walk_path(
        path: PathBuf,
        folder: music_folder::Model,
        tx: mpsc::Sender<WalkTask>,
    ) {
        tokio::task::spawn_blocking(move || {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let p = entry.path();
                // get ext and name
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                // filter only dir or audio files
                if !metadata.is_dir() &&!is_audio_file(ext.as_str()){
                    continue
                }
                let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();

                let mod_time: chrono::DateTime<chrono::Utc> = metadata.modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now())
                    .into();

                let task = WalkTask {
                    path: entry.path().to_string_lossy().replace('\\', "/"),
                    is_dir: entry.file_type().is_dir(),
                    name: name,
                    ext: ext,
                    size: metadata.len(),
                    mod_time,
                    folder: folder.clone(),
                };

                if tx.blocking_send(task).is_err() {
                    break;
                }
            }
        });
    }
}
