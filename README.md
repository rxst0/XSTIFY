# XSTIFY

A lightweight desktop music player for your Spotify Premium account. It searches songs, artists and albums, browses your Liked Songs and playlists, and plays everything inside one small window. The official Spotify app doesn't need to be running.

Built with [Tauri](https://tauri.app) (Rust + system WebView2) and Svelte. The release build is about 10 MB.

> XSTIFY is an unofficial, personal project. It is not affiliated with or endorsed by Spotify. It uses Spotify's official [Web API](https://developer.spotify.com/documentation/web-api) and [Web Playback SDK](https://developer.spotify.com/documentation/web-playback-sdk), and requires **Spotify Premium**.

## Features

- Search songs, artists and albums together as you type
- Artist pages (popular tracks, discography) and album pages
- Liked Songs and all your playlists, loaded as you scroll
- **Local Files**: add your own music (MP3, M4A, FLAC, WAV, OGG and more) with Add files, Add folder, or drag and drop. It's played by XSTIFY itself, not through Spotify. Files stay where they are; XSTIFY only remembers where to find them
- Player bar with play/pause, previous/next, seek and volume
- Shows the current song in Windows' media controls, so media keys, the volume flyout and Stream Deck "now playing" plugins work
- Themes: pick any main color and accent color, or choose a preset (Settings)
- Built-in 10-band equalizer with presets and an on/off switch. It only affects XSTIFY. It needs a silent audio output on your PC, such as Voicemod's "Dummy Output" (see [How the equalizer works](#how-the-equalizer-works))
- Built-in updates: when a new version is released, an **Update** button appears in the sidebar and installs it over the old version
- Keyboard shortcuts:

  | Keys | Action |
  |---|---|
  | `Ctrl+K` or `/` | Search |
  | `Space` | Play / pause |
  | `Ctrl+←` / `Ctrl+→` | Previous / next track |
  | `Alt+←` or mouse back button | Go back |

## Setup

### 1. Create a Spotify developer app (one time)

1. Go to the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard) and click **Create app**.
2. Add this **Redirect URI** exactly: `http://127.0.0.1:8899/callback`
3. Tick **Web API** and **Web Playback SDK**, then save.
4. Copy the app's **Client ID**. XSTIFY asks for it on first launch.

In development mode, Spotify only lets accounts listed under **User Management** use the app. Add your own account there if you get a 403 error.

### 2. Build and run

Requirements: [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, and on Windows the Visual Studio C++ Build Tools.

```sh
npm install
npm run tauri dev      # run with hot reload
npm run tauri build    # optimized build and installer
```

## How the equalizer works

Spotify's audio is DRM-protected inside its player frame, so apps can't process it directly. When the EQ is on:

1. Spotify's player inside XSTIFY sends its audio to a **silent output** (an output nobody listens to, such as Voicemod's "Dummy Output"), so the unprocessed sound isn't heard.
2. XSTIFY captures that audio using Windows' per-app capture, runs it through a 10-band equalizer, and plays it on your speakers.

Audio is processed in memory only and is never saved. If your PC has no silent output, the EQ explains this and stays off. Normal playback isn't affected.

To choose an audio output, the browser engine requires **microphone permission** for the frame asking. XSTIFY grants it only to Spotify's player (`https://sdk.scdn.co`). The microphone is opened for an instant to list outputs, then closed. XSTIFY never records.

## Privacy

- Sign-in uses OAuth with PKCE, so there is no client secret.
- Your refresh token and Client ID are stored in the OS credential store (Windows Credential Manager) and never in project files.
- XSTIFY talks only to Spotify.

## License

MIT
