//! Login: OAuth PKCE via librespot-oauth, refresh token kept in Windows Credential Manager.

use keyring::Entry;
use librespot_oauth::OAuthClientBuilder;
use std::time::{Duration, Instant};

const SERVICE: &str = "xstify";

pub const KEY_CLIENT_ID: &str = "client_id";
// Bump the suffix whenever SCOPES changes, so users re-approve once with the new scopes.
pub const KEY_API_REFRESH: &str = "api_refresh_v3";
/// Every token key ever used, so sign-out clears old ones too.
pub const ALL_TOKEN_KEYS: &[&str] = &["api_refresh", "api_refresh_v2", "api_refresh_v3", "play_refresh"];

/// Redirect URI the user registers on their Spotify developer app.
pub const API_PORT: u16 = 8899;
const REDIRECT: &str = "http://127.0.0.1:8899/callback";
const SCOPES: &[&str] = &[
    // Library + search
    "user-read-private",
    "user-library-read",
    "playlist-read-private",
    "playlist-read-collaborative",
    // Web Playback SDK (XSTIFY plays audio itself)
    "streaming",
    "user-read-email",
    "user-read-playback-state",
    "user-modify-playback-state",
];

const DONE_PAGE: &str = "<!doctype html><html><body style=\"font-family:system-ui;background:#121212;color:#eee;display:grid;place-items:center;height:100vh;margin:0\"><div style=\"text-align:center\"><h2>You're signed in to XSTIFY</h2><p>You can close this tab.</p></div></body></html>";

pub fn load(key: &str) -> Option<String> {
    Entry::new(SERVICE, key)
        .ok()?
        .get_password()
        .ok()
        .filter(|s| !s.is_empty())
}

pub fn save(key: &str, value: &str) {
    if let Ok(e) = Entry::new(SERVICE, key) {
        let _ = e.set_password(value);
    }
}

pub fn clear(key: &str) {
    if let Ok(e) = Entry::new(SERVICE, key) {
        let _ = e.delete_credential();
    }
}

pub struct Token {
    pub access: String,
    pub expires_at: Instant,
}

impl Token {
    pub fn is_fresh(&self) -> bool {
        Instant::now() + Duration::from_secs(60) < self.expires_at
    }
}

/// Get an access token. Uses the saved refresh token when possible (silent);
/// otherwise opens the browser for the user to approve.
pub async fn obtain(client_id: &str) -> Result<Token, String> {
    let client_id = client_id.to_string();
    let tok = tauri::async_runtime::spawn_blocking(move || {
        let client = OAuthClientBuilder::new(&client_id, REDIRECT, SCOPES.to_vec())
            .open_in_browser()
            .with_custom_message(DONE_PAGE)
            .build()
            .map_err(|e| e.to_string())?;
        if let Some(rt) = load(KEY_API_REFRESH) {
            if let Ok(t) = client.refresh_token(&rt) {
                return Ok(t);
            }
        }
        client.get_access_token().map_err(|e| {
            let e = e.to_string();
            if e.contains("/cancelled") {
                "Sign-in cancelled.".to_string()
            } else {
                format!("Login failed: {e}")
            }
        })
    })
    .await
    .map_err(|e| e.to_string())??;

    if !tok.refresh_token.is_empty() {
        save(KEY_API_REFRESH, &tok.refresh_token);
    }
    Ok(Token {
        access: tok.access_token,
        expires_at: tok.expires_at,
    })
}
