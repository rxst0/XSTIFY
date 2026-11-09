//! Windows media controls (SMTC): shows the current song to Windows, so media keys,
//! the volume flyout and Stream Deck "now playing" plugins can see and control XSTIFY.
//! Key presses are forwarded to the UI as "media-key" events.

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig, SeekDirection};
use std::{sync::Mutex, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};

pub struct Media(Mutex<Option<MediaControls>>);

pub fn setup(app: &AppHandle) {
    let controls = (|| {
        let window = app.get_webview_window("main")?;
        let hwnd = window.hwnd().ok()?.0 as *mut std::ffi::c_void;
        let mut c = MediaControls::new(PlatformConfig {
            display_name: "XSTIFY",
            dbus_name: "xstify",
            hwnd: Some(hwnd),
        })
        .ok()?;
        let handle = app.clone();
        c.attach(move |e| {
            let (action, ms): (&str, Option<u64>) = match e {
                MediaControlEvent::Play => ("play", None),
                MediaControlEvent::Pause | MediaControlEvent::Stop => ("pause", None),
                MediaControlEvent::Toggle => ("toggle", None),
                MediaControlEvent::Next => ("next", None),
                MediaControlEvent::Previous => ("previous", None),
                MediaControlEvent::SetPosition(MediaPosition(d)) => ("seek", Some(d.as_millis() as u64)),
                MediaControlEvent::Seek(SeekDirection::Forward) => ("forward", Some(10_000)),
                MediaControlEvent::Seek(SeekDirection::Backward) => ("backward", Some(10_000)),
                MediaControlEvent::SeekBy(SeekDirection::Forward, d) => ("forward", Some(d.as_millis() as u64)),
                MediaControlEvent::SeekBy(SeekDirection::Backward, d) => ("backward", Some(d.as_millis() as u64)),
                _ => return,
            };
            let _ = handle.emit("media-key", serde_json::json!({ "action": action, "ms": ms }));
        })
        .ok()?;
        Some(c)
    })();
    if controls.is_none() {
        log::warn!("Windows media controls unavailable");
    }
    app.manage(Media(Mutex::new(controls)));
}

/// New song: title, artists, album and cover art.
#[tauri::command]
pub fn media_track(media: State<'_, Media>, title: String, artist: String, album: String, cover: String, duration_ms: u64) {
    if let Some(c) = media.0.lock().unwrap().as_mut() {
        let _ = c.set_metadata(MediaMetadata {
            title: Some(&title),
            artist: Some(&artist),
            album: Some(&album),
            cover_url: (!cover.is_empty()).then_some(cover.as_str()),
            duration: Some(Duration::from_millis(duration_ms)),
        });
    }
}

/// Playing/paused state and position.
#[tauri::command]
pub fn media_state(media: State<'_, Media>, playing: bool, position_ms: u64) {
    if let Some(c) = media.0.lock().unwrap().as_mut() {
        let progress = Some(MediaPosition(Duration::from_millis(position_ms)));
        let _ = c.set_playback(if playing {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        });
    }
}
