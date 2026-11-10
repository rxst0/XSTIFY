//! Local Files: the user's own music files, played by XSTIFY itself (not through Spotify).
//! Only the list of file locations is saved (local.json in the app data folder);
//! the files themselves stay where they are. Each added file is allowed in the asset
//! protocol so the player can read it — nothing else on disk is exposed.

use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};

const AUDIO_EXTS: [&str; 9] = ["mp3", "m4a", "aac", "flac", "wav", "ogg", "opus", "webm", "wma"];

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalTrack {
    id: String,
    path: String,
    title: String,
    artist: String,
}

#[derive(Default)]
pub struct Local(Mutex<Vec<LocalTrack>>);

fn store_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("local.json"))
}

fn save(app: &AppHandle, list: &[LocalTrack]) -> Result<(), String> {
    let path = store_path(app).ok_or("No app data folder")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_vec(list).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn allow(app: &AppHandle, path: &str) {
    let _ = app.asset_protocol_scope().allow_file(path);
}

/// Load the saved list at startup and allow those files for playback.
pub fn setup(app: &AppHandle) {
    let list: Vec<LocalTrack> = store_path(app)
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    for t in &list {
        allow(app, &t.path);
    }
    app.manage(Local(Mutex::new(list)));
}

/// "Artist - Title.mp3" → ("Title", "Artist"); otherwise the file name is the title.
fn names(path: &Path) -> (String, String) {
    let stem = path.file_stem().map(|s| s.to_string_lossy().trim().to_string()).unwrap_or_default();
    match stem.split_once(" - ") {
        Some((artist, title)) if !artist.trim().is_empty() && !title.trim().is_empty() => {
            (title.trim().to_string(), artist.trim().to_string())
        }
        _ => (stem, "Unknown artist".into()),
    }
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .map(|e| AUDIO_EXTS.contains(&e.to_string_lossy().to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Audio files in `path` (a file, or a folder searched recursively), sorted by name.
fn collect(path: &Path, out: &mut Vec<PathBuf>, depth: u32) {
    if path.is_dir() {
        if depth > 8 {
            return;
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(path).into_iter().flatten().flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            collect(&p, out, depth + 1);
        }
    } else if is_audio(path) {
        out.push(path.to_path_buf());
    }
}

fn id_for(path: &str) -> String {
    let mut h = DefaultHasher::new();
    path.to_lowercase().hash(&mut h);
    format!("{:016x}", h.finish())
}

#[tauri::command]
pub fn local_list(local: State<'_, Local>) -> Vec<LocalTrack> {
    local.0.lock().unwrap().clone()
}

/// Add files and/or folders. Returns the updated list.
#[tauri::command]
pub fn local_add(app: AppHandle, local: State<'_, Local>, paths: Vec<String>) -> Result<Vec<LocalTrack>, String> {
    let mut found = Vec::new();
    for p in &paths {
        collect(Path::new(p), &mut found, 0);
    }
    let mut list = local.0.lock().unwrap();
    for f in found {
        let path = f.to_string_lossy().to_string();
        let id = id_for(&path);
        if list.iter().any(|t| t.id == id) {
            continue; // already added
        }
        let (title, artist) = names(&f);
        allow(&app, &path);
        list.push(LocalTrack { id, path, title, artist });
    }
    save(&app, &list)?;
    Ok(list.clone())
}

#[tauri::command]
pub fn local_remove(app: AppHandle, local: State<'_, Local>, id: String) -> Result<Vec<LocalTrack>, String> {
    let mut list = local.0.lock().unwrap();
    list.retain(|t| t.id != id);
    save(&app, &list)?;
    Ok(list.clone())
}
