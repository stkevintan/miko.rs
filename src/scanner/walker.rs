use walkdir::WalkDir;
use crate::models::music_folder;
use tokio::sync::mpsc;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WalkTask {
    pub path: String,
    pub is_dir: bool,
    pub name: String,
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
        tokio::spawn(async move {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                
                let mod_time: chrono::DateTime<chrono::Utc> = metadata.modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now())
                    .into();

                let task = WalkTask {
                    path: entry.path().to_string_lossy().replace('\\', "/"),
                    is_dir: entry.file_type().is_dir(),
                    name: entry.file_name().to_string_lossy().into_owned(),
                    size: metadata.len(),
                    mod_time,
                    folder: folder.clone(),
                };

                if tx.send(task).await.is_err() {
                    break;
                }
            }
        });
    }
}
