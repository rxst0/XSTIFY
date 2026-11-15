mod auth;
mod eq;
mod ext;
mod local;
mod media;

use serde::Serialize;
use serde_json::Value;
use tauri::State;
use tokio::sync::Mutex;

struct AppState {
    http: reqwest::Client,
    api: Mutex<Option<auth::Token>>,
}

#[derive(Serialize)]
struct Status {
    has_client_id: bool,
    has_saved_login: bool,
}

#[tauri::command]
fn status() -> Status {
    Status {
        has_client_id: auth::load(auth::KEY_CLIENT_ID).is_some(),
        has_saved_login: auth::load(auth::KEY_API_REFRESH).is_some(),
    }
}

/// Sign in: silently if a refresh token is saved, otherwise via the browser.
#[tauri::command]
async fn login(state: State<'_, AppState>, client_id: Option<String>) -> Result<(), String> {
    if let Some(cid) = client_id.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        auth::save(auth::KEY_CLIENT_ID, cid);
    }
    api_token(&state, true).await.map(|_| ())
}

/// Abort a pending browser sign-in by poking the local callback listener,
/// which makes it return an error instead of waiting forever.
#[tauri::command]
fn cancel_login() {
    use std::io::Write;
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], auth::API_PORT));
    if let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
        let _ = s.write_all(b"GET /cancelled HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    }
}

#[tauri::command]
async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    *state.api.lock().await = None;
    for k in auth::ALL_TOKEN_KEYS {
        auth::clear(k);
    }
    Ok(())
}

/// Fresh access token for Spotify's Web Playback SDK.
#[tauri::command]
async fn access_token(state: State<'_, AppState>) -> Result<String, String> {
    api_token(&state, false).await
}

async fn api_token(state: &AppState, force: bool) -> Result<String, String> {
    let mut guard = state.api.lock().await;
    if let Some(t) = guard.as_ref().filter(|t| !force && t.is_fresh()) {
        return Ok(t.access.clone());
    }
    let cid = auth::load(auth::KEY_CLIENT_ID).ok_or("Enter your Spotify client ID first")?;
    let t = auth::obtain(&cid).await?;
    let access = t.access.clone();
    *guard = Some(t);
    Ok(access)
}

/// GET a Spotify Web API path (e.g. "/search?q=..") or a full "next" page URL.
#[tauri::command]
async fn api_get(state: State<'_, AppState>, path: String) -> Result<Value, String> {
    api_call(&state, reqwest::Method::GET, path, None).await
}

/// PUT/POST to the Web API (e.g. start playback on the XSTIFY device).
#[tauri::command]
async fn api_send(state: State<'_, AppState>, method: String, path: String, body: Option<Value>) -> Result<Value, String> {
    let method = match method.as_str() {
        "PUT" => reqwest::Method::PUT,
        "POST" => reqwest::Method::POST,
        "DELETE" => reqwest::Method::DELETE,
        _ => return Err("Unsupported method".into()),
    };
    api_call(&state, method, path, body).await
}

async fn api_call(state: &AppState, method: reqwest::Method, path: String, body: Option<Value>) -> Result<Value, String> {
    const BASE: &str = "https://api.spotify.com/v1";
    let url = if path.starts_with('/') {
        format!("{BASE}{path}")
    } else if path.starts_with(BASE) {
        path
    } else {
        return Err("Invalid API path".into());
    };

    for attempt in 0..3 {
        let token = api_token(state, false).await?;
        let mut req = state.http.request(method.clone(), &url).bearer_auth(token);
        req = match &body {
            Some(b) => req.json(b),
            None if method != reqwest::Method::GET => req.header("content-length", "0"),
            None => req,
        };
        let res = req
            .send()
            .await
            .map_err(|_| "Can't reach Spotify. Check your connection.".to_string())?;
        let status = res.status().as_u16();
        if !(200..300).contains(&status) {
            log::warn!("{method} {url} -> {status}");
        }
        match status {
            401 if attempt < 2 => {
                api_token(state, true).await?; // token expired early: refresh and retry
                continue;
            }
            // Spotify's servers occasionally hiccup (502/503/504): wait briefly and retry.
            502..=504 if attempt < 2 => {
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                continue;
            }
            204 => return Ok(Value::Null),
            429 => return Err("Spotify is rate-limiting requests. Try again in a moment.".into()),
            s if !(200..300).contains(&s) => {
                let body = res.text().await.unwrap_or_default();
                log::warn!("  body: {body}");
                return Err(format!("Spotify error {s}: {body}"));
            }
            _ => {
                let text = res.text().await.map_err(|e| e.to_string())?;
                return Ok(serde_json::from_str(&text).unwrap_or(Value::Null));
            }
        }
    }
    Err("Session expired. Please sign in again.".into())
}

/// Grant microphone permission to Spotify's player frame only (and nothing else).
/// It's needed solely so the player can switch its audio output for the built-in EQ;
/// the frame script closes the mic immediately after the one-time check.
fn allow_player_mic(window: &tauri::WebviewWindow) {
    use webview2_com::{
        Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE, COREWEBVIEW2_PERMISSION_STATE_ALLOW},
        PermissionRequestedEventHandler,
    };
    let _ = window.with_webview(|wv| unsafe {
        let Ok(core) = wv.controller().CoreWebView2() else { return };
        let mut token = Default::default();
        let _ = core.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(|_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                args.PermissionKind(&mut kind)?;
                let mut uri = windows::core::PWSTR::null();
                args.Uri(&mut uri)?;
                let uri = webview2_com::take_pwstr(uri);
                // Spotify's player frame, plus XSTIFY's own page (for routing Local Files
                // audio when the EQ is on). Release builds use tauri.localhost; dev uses localhost:1420.
                let allowed = ["https://sdk.scdn.co/", "http://tauri.localhost/", "http://localhost:1420/"];
                if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE && allowed.iter().any(|a| uri.starts_with(a)) {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                }
                Ok(())
            })),
            &mut token,
        );
    });
}

/// Free mode: open (or reuse) a window with Spotify's official web player at `url`.
/// The window gets no access to XSTIFY's commands (it isn't in any capability).
#[tauri::command]
fn open_web_player(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::Manager;
    const LABEL: &str = "spotify-web";
    let parsed: tauri::Url = url.parse().map_err(|_| "Invalid link".to_string())?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("open.spotify.com") {
        return Err("Only Spotify's web player can be opened.".into());
    }
    if let Some(w) = app.get_webview_window(LABEL) {
        w.navigate(parsed).map_err(|e| e.to_string())?;
        let _ = w.unminimize();
        let _ = w.set_focus();
        return Ok(());
    }
    // Must match the main window's browser arguments, or WebView2 refuses to start
    // a second window that shares the same profile.
    let args = app.config().app.windows[0].additional_browser_args.clone().unwrap_or_default();
    let mut b = tauri::WebviewWindowBuilder::new(&app, LABEL, tauri::WebviewUrl::External(parsed))
        .title("Spotify Web Player · XSTIFY")
        .inner_size(1100.0, 760.0)
        .min_inner_size(720.0, 500.0);
    if !args.is_empty() {
        b = b.additional_browser_args(&args);
    }
    b.build().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("xstify_lib=info")).init();
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            http: reqwest::Client::new(),
            api: Mutex::new(None),
        })
        .manage(eq::Eq::default())
        .setup(|app| {
            // The window is built here (not from config alone) so the frame script can be added.
            let cfg = app.config().app.windows[0].clone();
            let window = tauri::WebviewWindowBuilder::from_config(app.handle(), &cfg)?
                .initialization_script_for_all_frames(include_str!("frame_script.js"))
                .build()?;
            allow_player_mic(&window);
            media::setup(app.handle());
            local::setup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status, login, cancel_login, logout, access_token, api_get, api_send, open_web_player,
            media::media_track, media::media_state,
            eq::eq_status, eq::eq_start, eq::eq_gains, eq::eq_stop,
            local::local_list, local::local_add, local::local_remove,
            ext::ext_list, ext::ext_add, ext::ext_remove, ext::ext_set_enabled, ext::ext_code
        ])
        .run(tauri::generate_context!())
        .expect("error while running XSTIFY");
}
