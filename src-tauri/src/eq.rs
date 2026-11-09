//! Built-in 10-band equalizer.
//!
//! Spotify's player audio is DRM-protected inside its own frame, so XSTIFY can't touch it
//! directly. Instead, when the EQ is on:
//!   1. Spotify's player frame sends its audio to a silent output (e.g. "Dummy Output"),
//!      see `frame_script.js`, so the unprocessed sound isn't heard.
//!   2. We capture that audio with Windows' per-app loopback (WebView2's process tree only).
//!   3. Run it through 10 peaking filters and play it on the default speakers.
//! Audio is processed in memory only and never stored.

use serde::Serialize;
use std::{
    mem::size_of,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::State;
use windows::{
    core::{implement, Interface, Ref, Result as WinResult, HRESULT},
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Foundation::CloseHandle,
        Media::Audio::*,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, IAgileObject, IAgileObject_Impl, StructuredStorage::PROPVARIANT, BLOB,
                CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
            },
            Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
            Variant::VT_BLOB,
        },
    },
};

const RATE: u32 = 48_000;
const BANDS: [f32; 10] = [31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
/// Words that identify an output nobody listens to.
const SILENT_HINTS: [&str; 4] = ["dummy", "null", "virtual", "cable"];

pub struct Eq {
    running: Mutex<Option<Running>>,
    gains: Arc<Mutex<[f32; 10]>>,
    dirty: Arc<AtomicBool>,
}

struct Running {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl Default for Eq {
    fn default() -> Self {
        Self { running: Mutex::new(None), gains: Arc::new(Mutex::new([0.0; 10])), dirty: Arc::new(AtomicBool::new(true)) }
    }
}

#[derive(Serialize)]
pub struct EqStatus {
    available: bool,
    output: Option<String>,
}

// ---------------------------------------------------------------- commands

#[tauri::command]
pub async fn eq_status() -> EqStatus {
    let output = tauri::async_runtime::spawn_blocking(silent_output).await.ok().flatten();
    EqStatus { available: output.is_some(), output }
}

/// Start the engine. Returns the name of the silent output Spotify's player should use.
#[tauri::command]
pub async fn eq_start(eq: State<'_, Eq>, gains: Vec<f32>) -> Result<String, String> {
    set_gains(&eq, &gains);
    let output = tauri::async_runtime::spawn_blocking(silent_output)
        .await
        .ok()
        .flatten()
        .ok_or("No silent audio output found. The equalizer needs one, such as Voicemod's \"Dummy Output\".")?;
    let browser = webview_browser_pid().ok_or("Couldn't find XSTIFY's player process.")?;

    let mut running = eq.running.lock().unwrap();
    if running.is_none() {
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let (g, d, s) = (eq.gains.clone(), eq.dirty.clone(), stop.clone());
        let thread = thread::Builder::new()
            .name("xstify-eq".into())
            .spawn(move || {
                if let Err(e) = engine(browser, g, d, s, &tx) {
                    let _ = tx.send(Err(e.to_string()));
                }
            })
            .map_err(|e| e.to_string())?;
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => *running = Some(Running { stop, thread }),
            Ok(Err(e)) => return Err(format!("Equalizer failed to start: {e}")),
            Err(_) => return Err("Equalizer failed to start (timed out).".into()),
        }
    }
    Ok(output)
}

#[tauri::command]
pub fn eq_gains(eq: State<'_, Eq>, gains: Vec<f32>) {
    set_gains(&eq, &gains);
}

#[tauri::command]
pub fn eq_stop(eq: State<'_, Eq>) {
    if let Some(r) = eq.running.lock().unwrap().take() {
        r.stop.store(true, Ordering::SeqCst);
        let _ = r.thread.join();
    }
}

fn set_gains(eq: &Eq, gains: &[f32]) {
    let mut g = eq.gains.lock().unwrap();
    for (i, v) in g.iter_mut().enumerate() {
        *v = gains.get(i).copied().unwrap_or(0.0).clamp(-12.0, 12.0);
    }
    eq.dirty.store(true, Ordering::SeqCst);
}

// ---------------------------------------------------------------- devices & processes

/// Friendly name of an active output that's meant to be silent (and isn't the default).
fn silent_output() -> Option<String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let default_id = en.GetDefaultAudioEndpoint(eRender, eConsole).and_then(|d| d.GetId()).ok().map(|p| p.to_string().unwrap_or_default());
        let list = en.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE).ok()?;
        let mut best: Option<(usize, String)> = None;
        for i in 0..list.GetCount().ok()? {
            let Ok(dev) = list.Item(i) else { continue };
            let id = dev.GetId().ok().map(|p| p.to_string().unwrap_or_default());
            if id.is_some() && id == default_id {
                continue;
            }
            let Ok(store) = dev.OpenPropertyStore(STGM_READ) else { continue };
            let Ok(name) = store.GetValue(&PKEY_Device_FriendlyName) else { continue };
            let name = name.to_string();
            let lower = name.to_lowercase();
            if let Some(rank) = SILENT_HINTS.iter().position(|h| lower.contains(h)) {
                if best.as_ref().is_none_or(|(r, _)| rank < *r) {
                    best = Some((rank, name));
                }
            }
        }
        best.map(|(_, n)| n)
    }
}

/// The WebView2 browser process started by XSTIFY (its children render the audio).
fn webview_browser_pid() -> Option<u32> {
    let me = std::process::id();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut e = PROCESSENTRY32W { dwSize: size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut found = None;
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                let len = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                let name = String::from_utf16_lossy(&e.szExeFile[..len]);
                if e.th32ParentProcessID == me && name.eq_ignore_ascii_case("msedgewebview2.exe") {
                    found = Some(e.th32ProcessID);
                    break;
                }
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        found
    }
}

// ---------------------------------------------------------------- audio engine

#[implement(IActivateAudioInterfaceCompletionHandler, IAgileObject)]
struct Activated(mpsc::Sender<()>);
impl IActivateAudioInterfaceCompletionHandler_Impl for Activated_Impl {
    fn ActivateCompleted(&self, _op: Ref<IActivateAudioInterfaceAsyncOperation>) -> WinResult<()> {
        let _ = self.0.send(());
        Ok(())
    }
}
impl IAgileObject_Impl for Activated_Impl {}

fn format() -> WAVEFORMATEX {
    WAVEFORMATEX {
        wFormatTag: 3, // IEEE float
        nChannels: 2,
        nSamplesPerSec: RATE,
        nAvgBytesPerSec: RATE * 8,
        nBlockAlign: 8,
        wBitsPerSample: 32,
        cbSize: 0,
    }
}

/// Capture client for everything the given process tree plays.
unsafe fn loopback_client(pid: u32) -> WinResult<IAudioClient> {
    let mut params = AUDIOCLIENT_ACTIVATION_PARAMS {
        ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
        Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
            ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                TargetProcessId: pid,
                ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_INCLUDE_TARGET_PROCESS_TREE,
            },
        },
    };
    // ManuallyDrop: PROPVARIANT's Drop calls PropVariantClear, which would try to free
    // `params` (a stack value we only lend it) and corrupt the heap.
    let mut pv = std::mem::ManuallyDrop::new(PROPVARIANT::default());
    unsafe {
        let inner = &mut *pv.Anonymous.Anonymous;
        inner.vt = VT_BLOB;
        inner.Anonymous.blob = BLOB {
            cbSize: size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
            pBlobData: &mut params as *mut _ as *mut u8,
        };
        let (tx, rx) = mpsc::channel();
        let handler: IActivateAudioInterfaceCompletionHandler = Activated(tx).into();
        let op = ActivateAudioInterfaceAsync(VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, &IAudioClient::IID, Some(&*pv), &handler)?;
        rx.recv_timeout(Duration::from_secs(5)).map_err(|_| windows::core::Error::empty())?;
        let mut hr = HRESULT(0);
        let mut unk = None;
        op.GetActivateResult(&mut hr, &mut unk)?;
        hr.ok()?;
        let client: IAudioClient = unk.ok_or_else(windows::core::Error::empty)?.cast()?;
        client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
            1_000_000, // 100 ms buffer (100 ns units)
            0,
            &format(),
            None,
        )?;
        Ok(client)
    }
}

/// Render client on the default speakers.
unsafe fn speaker_client() -> WinResult<IAudioClient> {
    unsafe {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let client: IAudioClient = dev.Activate(CLSCTX_ALL, None)?;
        client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
            1_000_000,
            0,
            &format(),
            None,
        )?;
        Ok(client)
    }
}

fn engine(
    browser: u32,
    gains: Arc<Mutex<[f32; 10]>>,
    dirty: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    ready: &mpsc::Sender<Result<(), String>>,
) -> WinResult<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let cap_client = loopback_client(browser)?;
        let out_client = speaker_client()?;
        let cap: IAudioCaptureClient = cap_client.GetService()?;
        let out: IAudioRenderClient = out_client.GetService()?;
        let out_size = out_client.GetBufferSize()?;

        // Small cushion of silence so playback doesn't stutter (20 ms).
        let pre = (RATE / 50).min(out_size);
        out.GetBuffer(pre)?;
        out.ReleaseBuffer(pre, AUDCLNT_BUFFERFLAGS_SILENT.0 as u32)?;

        cap_client.Start()?;
        out_client.Start()?;
        let _ = ready.send(Ok(()));

        let mut eq = Equalizer::default();
        let mut scratch: Vec<f32> = Vec::new();
        while !stop.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(3));
            if dirty.swap(false, Ordering::SeqCst) {
                eq.set(&gains.lock().unwrap());
            }
            loop {
                if cap.GetNextPacketSize()? == 0 {
                    break;
                }
                let mut data = std::ptr::null_mut();
                let (mut frames, mut flags) = (0u32, 0u32);
                cap.GetBuffer(&mut data, &mut frames, &mut flags, None, None)?;
                let n = frames as usize * 2;
                scratch.clear();
                if flags & (AUDCLNT_BUFFERFLAGS_SILENT.0 as u32) != 0 {
                    scratch.resize(n, 0.0);
                } else {
                    scratch.extend_from_slice(std::slice::from_raw_parts(data as *const f32, n));
                }
                cap.ReleaseBuffer(frames)?;
                eq.process(&mut scratch);

                // Write what fits; if the speakers fell behind, drop the excess.
                let room = out_size - out_client.GetCurrentPadding()?;
                let w = frames.min(room);
                if w > 0 {
                    let dst = out.GetBuffer(w)? as *mut f32;
                    std::ptr::copy_nonoverlapping(scratch.as_ptr(), dst, w as usize * 2);
                    out.ReleaseBuffer(w, 0)?;
                }
            }
        }
        let _ = cap_client.Stop();
        let _ = out_client.Stop();
        Ok(())
    }
}

// ---------------------------------------------------------------- filters

/// One peaking EQ biquad (RBJ cookbook), stereo state.
#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: [f32; 2],
    x2: [f32; 2],
    y1: [f32; 2],
    y2: [f32; 2],
}

impl Biquad {
    fn peaking(freq: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w = 2.0 * std::f32::consts::PI * freq / RATE as f32;
        let alpha = w.sin() / (2.0 * std::f32::consts::SQRT_2); // Q ≈ 1.41: one octave wide
        let cos = w.cos();
        let a0 = 1.0 + alpha / a;
        Self {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos) / a0,
            a2: (1.0 - alpha / a) / a0,
            ..Default::default()
        }
    }

    #[inline]
    fn tick(&mut self, c: usize, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1[c] + self.b2 * self.x2[c] - self.a1 * self.y1[c] - self.a2 * self.y2[c];
        self.x2[c] = self.x1[c];
        self.x1[c] = x;
        self.y2[c] = self.y1[c];
        self.y1[c] = y;
        y
    }
}

#[derive(Default)]
struct Equalizer {
    filters: Vec<Biquad>,
    preamp: f32,
}

impl Equalizer {
    fn set(&mut self, gains: &[f32; 10]) {
        // Keep filter state for bands that stay active, so slider moves don't click.
        let old = std::mem::take(&mut self.filters);
        self.filters = BANDS
            .iter()
            .zip(gains)
            .filter(|(_, g)| g.abs() >= 0.05)
            .map(|(&f, &g)| {
                let mut b = Biquad::peaking(f, g);
                if let Some(prev) = old.iter().find(|p| p.b1 == b.b1) {
                    (b.x1, b.x2, b.y1, b.y2) = (prev.x1, prev.x2, prev.y1, prev.y2);
                }
                b
            })
            .collect();
        // Turn the overall level down by the biggest boost so loud songs don't clip.
        let max_boost = gains.iter().copied().fold(0.0_f32, f32::max);
        self.preamp = 10f32.powf(-max_boost / 20.0);
    }

    fn process(&mut self, buf: &mut [f32]) {
        if self.filters.is_empty() {
            return;
        }
        for frame in buf.chunks_exact_mut(2) {
            for (c, s) in frame.iter_mut().enumerate() {
                let mut v = *s * self.preamp;
                for f in &mut self.filters {
                    v = f.tick(c, v);
                }
                *s = v.clamp(-1.0, 1.0);
            }
        }
    }
}
