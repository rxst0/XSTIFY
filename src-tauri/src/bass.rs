//! Bass boost via Equalizer APO (system-wide audio equalizer).
//! XSTIFY can't process Spotify's DRM-protected audio itself, so the boost is applied
//! by Equalizer APO. We manage one small config file and include it from config.txt;
//! Equalizer APO picks up changes instantly.

use serde::Serialize;
use std::{fs, os::windows::process::CommandExt, path::PathBuf, process::Command};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const FILE: &str = "xstify-bass.txt";
const INCLUDE: &str = "Include: xstify-bass.txt";
const DOWNLOAD: &str = "https://sourceforge.net/projects/equalizerapo/";

// Low-shelf +6 dB below ~100 Hz, with -6 dB preamp so loud tracks don't clip.
const ON: &str = "# Managed by XSTIFY: bass boost ON\nPreamp: -6 dB\nFilter: ON LSC Fc 100 Hz Gain 6 dB\n";
const OFF: &str = "# Managed by XSTIFY: bass boost OFF\n";

#[derive(Serialize)]
pub struct BassStatus {
    installed: bool,
    enabled: bool,
}

/// Equalizer APO's config folder (from its registry key, else the default location).
fn config_dir() -> Option<PathBuf> {
    let out = Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\EqualizerAPO", "/v", "ConfigPath"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok();
    let from_reg = out.and_then(|o| {
        String::from_utf8_lossy(&o.stdout)
            .lines()
            .find_map(|l| l.split_once("REG_SZ").map(|(_, p)| PathBuf::from(p.trim())))
    });
    from_reg
        .or_else(|| Some(PathBuf::from(r"C:\Program Files\EqualizerAPO\config")))
        .filter(|p| p.join("config.txt").exists())
}

#[tauri::command]
pub fn bass_status() -> BassStatus {
    match config_dir() {
        Some(dir) => BassStatus {
            installed: true,
            enabled: fs::read_to_string(dir.join(FILE)).is_ok_and(|s| s.contains("Filter: ON")),
        },
        None => BassStatus { installed: false, enabled: false },
    }
}

#[tauri::command]
pub fn bass_set(enabled: bool) -> Result<(), String> {
    let dir = config_dir().ok_or("NOT_INSTALLED")?;
    let denied = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "Windows blocked XSTIFY from changing Equalizer APO's settings.".to_string()
        } else {
            e.to_string()
        }
    };
    fs::write(dir.join(FILE), if enabled { ON } else { OFF }).map_err(denied)?;

    // Make sure Equalizer APO's main config loads our file (once; other settings untouched).
    let main = dir.join("config.txt");
    let cfg = fs::read_to_string(&main).map_err(|e| e.to_string())?;
    if !cfg.lines().any(|l| l.trim().eq_ignore_ascii_case(INCLUDE)) {
        let sep = if cfg.ends_with('\n') || cfg.is_empty() { "" } else { "\r\n" };
        fs::write(&main, format!("{cfg}{sep}{INCLUDE}\r\n")).map_err(denied)?;
    }
    Ok(())
}

/// Open the Equalizer APO download page in the browser.
#[tauri::command]
pub fn bass_get_eq() {
    let _ = Command::new("cmd")
        .args(["/C", "start", "", DOWNLOAD])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
