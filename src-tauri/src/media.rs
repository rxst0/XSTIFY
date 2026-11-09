//! Windows media controls (SMTC): tells Windows what's playing, so media keys, the volume
//! flyout and Stream Deck "now playing" plugins can show and control XSTIFY.
//! Button presses are forwarded to the UI as "media-key" events.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, OnceLock,
};
use tauri::{AppHandle, Emitter, Manager, State};
use windows::{
    core::{factory, HSTRING},
    Foundation::{TimeSpan, TypedEventHandler},
    Media::{
        MediaPlaybackStatus, MediaPlaybackType, PlaybackPositionChangeRequestedEventArgs, SystemMediaTransportControls,
        SystemMediaTransportControlsButton, SystemMediaTransportControlsButtonPressedEventArgs,
        SystemMediaTransportControlsTimelineProperties,
    },
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream, RandomAccessStreamReference},
    Win32::{Foundation::HWND, System::WinRT::ISystemMediaTransportControlsInterop},
};

pub struct Media(Mutex<Option<SystemMediaTransportControls>>);

static HTTP: OnceLock<reqwest::Client> = OnceLock::new();

const fn span(ms: u64) -> TimeSpan {
    TimeSpan { Duration: (ms * 10_000) as i64 } // 100 ns units
}

fn emit(app: &AppHandle, action: &str, ms: Option<u64>) {
    let _ = app.emit("media-key", serde_json::json!({ "action": action, "ms": ms }));
}

fn create(app: &AppHandle) -> windows::core::Result<SystemMediaTransportControls> {
    let window = app.get_webview_window("main").ok_or_else(windows::core::Error::empty)?;
    let hwnd = window.hwnd().map_err(|_| windows::core::Error::empty())?;
    let interop = factory::<SystemMediaTransportControls, ISystemMediaTransportControlsInterop>()?;
    let smtc: SystemMediaTransportControls = unsafe { interop.GetForWindow(HWND(hwnd.0 as _)) }?;

    smtc.SetIsEnabled(true)?;
    smtc.SetIsPlayEnabled(true)?;
    smtc.SetIsPauseEnabled(true)?;
    smtc.SetIsStopEnabled(true)?;
    smtc.SetIsNextEnabled(true)?;
    smtc.SetIsPreviousEnabled(true)?;
    // Without this, Windows ignores title/artist/cover.
    smtc.DisplayUpdater()?.SetType(MediaPlaybackType::Music)?;

    let a = app.clone();
    smtc.ButtonPressed(&TypedEventHandler::<SystemMediaTransportControls, SystemMediaTransportControlsButtonPressedEventArgs>::new(
        move |_, args| {
            if let Some(args) = args.as_ref() {
                let action = match args.Button()? {
                    SystemMediaTransportControlsButton::Play => "play",
                    SystemMediaTransportControlsButton::Pause | SystemMediaTransportControlsButton::Stop => "pause",
                    SystemMediaTransportControlsButton::Next => "next",
                    SystemMediaTransportControlsButton::Previous => "previous",
                    SystemMediaTransportControlsButton::FastForward => "forward",
                    SystemMediaTransportControlsButton::Rewind => "backward",
                    _ => return Ok(()),
                };
                emit(&a, action, None);
            }
            Ok(())
        },
    ))?;

    let a = app.clone();
    smtc.PlaybackPositionChangeRequested(&TypedEventHandler::<SystemMediaTransportControls, PlaybackPositionChangeRequestedEventArgs>::new(
        move |_, args| {
            if let Some(args) = args.as_ref() {
                let ms = args.RequestedPlaybackPosition()?.Duration.max(0) as u64 / 10_000;
                emit(&a, "seek", Some(ms));
            }
            Ok(())
        },
    ))?;

    Ok(smtc)
}

pub fn setup(app: &AppHandle) {
    let smtc = create(app).map_err(|e| log::warn!("Windows media controls unavailable: {e}")).ok();
    app.manage(Media(Mutex::new(smtc)));
}

static TRACK_SEQ: AtomicU64 = AtomicU64::new(0);

/// Cover art as an in-memory image. (A URL thumbnail shows in some places, but readers
/// such as Stream Deck plugins get empty data from it, so we hand over the actual bytes.)
fn thumbnail(bytes: &[u8]) -> windows::core::Result<RandomAccessStreamReference> {
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(bytes)?;
    writer.StoreAsync()?.join()?;
    writer.FlushAsync()?.join()?;
    writer.DetachStream()?;
    stream.Seek(0)?;
    RandomAccessStreamReference::CreateFromStream(&stream)
}

/// New song: title, artists, album, cover art and length.
#[tauri::command]
pub async fn media_track(
    media: State<'_, Media>,
    title: String,
    artist: String,
    album: String,
    cover: String,
    duration_ms: u64,
) -> Result<(), String> {
    let seq = TRACK_SEQ.fetch_add(1, Ordering::SeqCst) + 1;
    // Only Spotify's image CDN, so this can't be used to fetch arbitrary URLs.
    let bytes = if cover.starts_with("https://i.scdn.co/") {
        let http = HTTP.get_or_init(reqwest::Client::new);
        match http.get(&cover).send().await {
            Ok(r) if r.status().is_success() => r.bytes().await.ok(),
            _ => None,
        }
    } else {
        None
    };
    if seq != TRACK_SEQ.load(Ordering::SeqCst) {
        return Ok(()); // a newer song started while the cover downloaded
    }

    let guard = media.0.lock().unwrap();
    let Some(smtc) = guard.as_ref() else { return Ok(()) };
    let result = (|| -> windows::core::Result<()> {
        let du = smtc.DisplayUpdater()?;
        du.SetType(MediaPlaybackType::Music)?;
        let mp = du.MusicProperties()?;
        mp.SetTitle(&HSTRING::from(&title))?;
        mp.SetArtist(&HSTRING::from(&artist))?;
        mp.SetAlbumTitle(&HSTRING::from(&album))?;
        if let Some(b) = bytes.as_deref() {
            du.SetThumbnail(&thumbnail(b)?)?;
        }
        du.Update()?;

        let tl = SystemMediaTransportControlsTimelineProperties::new()?;
        tl.SetStartTime(span(0))?;
        tl.SetMinSeekTime(span(0))?;
        tl.SetEndTime(span(duration_ms))?;
        tl.SetMaxSeekTime(span(duration_ms))?;
        tl.SetPosition(span(0))?;
        smtc.UpdateTimelineProperties(&tl)
    })();
    if let Err(e) = result {
        log::warn!("media_track failed: {e}");
    }
    Ok(())
}

/// Playing/paused state and position.
#[tauri::command]
pub fn media_state(media: State<'_, Media>, playing: bool, position_ms: u64, duration_ms: u64) {
    let guard = media.0.lock().unwrap();
    let Some(smtc) = guard.as_ref() else { return };
    let result = (|| -> windows::core::Result<()> {
        smtc.SetPlaybackStatus(if playing { MediaPlaybackStatus::Playing } else { MediaPlaybackStatus::Paused })?;
        let tl = SystemMediaTransportControlsTimelineProperties::new()?;
        tl.SetStartTime(span(0))?;
        tl.SetMinSeekTime(span(0))?;
        tl.SetEndTime(span(duration_ms))?;
        tl.SetMaxSeekTime(span(duration_ms))?;
        tl.SetPosition(span(position_ms.min(duration_ms)))?;
        smtc.UpdateTimelineProperties(&tl)
    })();
    if let Err(e) = result {
        log::warn!("media_state failed: {e}");
    }
}
