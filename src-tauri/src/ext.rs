//! Extensions: Spicetify-style .js files the user adds. They're copied into the app data
//! folder (extensions/) and run by the compatibility layer in src/lib/spicetify.svelte.ts.
//! Only file names are accepted from the UI, never paths, so nothing outside that folder
//! can be read or deleted.

use serde::Serialize;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct ExtInfo {
    name: String,
    enabled: bool,
}

fn dir(app: &AppHandle) -> Result<PathBuf, String> {
    let d = app.path().app_data_dir().map_err(|e| e.to_string())?.join("extensions");
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    Ok(d)
}

/// Names of disabled extensions (everything else is on).
fn disabled(app: &AppHandle) -> Vec<String> {
    dir(app)
        .ok()
        .and_then(|d| fs::read(d.join("disabled.json")).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn save_disabled(app: &AppHandle, list: &[String]) -> Result<(), String> {
    let path = dir(app)?.join("disabled.json");
    fs::write(path, serde_json::to_vec(list).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn valid_name(name: &str) -> Result<(), String> {
    let ok = name.ends_with(".js") && !name.contains(['/', '\\', ':']) && !name.starts_with('.');
    if ok { Ok(()) } else { Err("Invalid extension name".into()) }
}

#[tauri::command]
pub fn ext_list(app: AppHandle) -> Result<Vec<ExtInfo>, String> {
    let off = disabled(&app);
    let mut names: Vec<String> = fs::read_dir(dir(&app)?)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".js"))
        .collect();
    names.sort_by_key(|n| n.to_lowercase());
    Ok(names.into_iter().map(|n| ExtInfo { enabled: !off.contains(&n), name: n }).collect())
}

/// Copy the chosen .js files into the extensions folder (on by default).
#[tauri::command]
pub fn ext_add(app: AppHandle, paths: Vec<String>) -> Result<Vec<ExtInfo>, String> {
    let d = dir(&app)?;
    for p in paths {
        let src = PathBuf::from(&p);
        let Some(name) = src.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
        if valid_name(&name).is_err() {
            continue;
        }
        fs::copy(&src, d.join(&name)).map_err(|e| format!("Couldn't add {name}: {e}"))?;
    }
    ext_list(app)
}

#[tauri::command]
pub fn ext_remove(app: AppHandle, name: String) -> Result<Vec<ExtInfo>, String> {
    valid_name(&name)?;
    let _ = fs::remove_file(dir(&app)?.join(&name));
    let mut off = disabled(&app);
    off.retain(|n| n != &name);
    save_disabled(&app, &off)?;
    ext_list(app)
}

#[tauri::command]
pub fn ext_set_enabled(app: AppHandle, name: String, enabled: bool) -> Result<Vec<ExtInfo>, String> {
    valid_name(&name)?;
    let mut off = disabled(&app);
    off.retain(|n| n != &name);
    if !enabled {
        off.push(name);
    }
    save_disabled(&app, &off)?;
    ext_list(app)
}

/// Source code of an extension, for the compatibility layer to run.
#[tauri::command]
pub fn ext_code(app: AppHandle, name: String) -> Result<String, String> {
    valid_name(&name)?;
    fs::read_to_string(dir(&app)?.join(&name)).map_err(|e| e.to_string())
}
