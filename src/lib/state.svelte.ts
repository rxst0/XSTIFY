import { invoke } from "@tauri-apps/api/core";
import { errMsg, type Track } from "./api";

// ---------- Navigation ----------

export type View =
  | { name: "search" }
  | { name: "liked" }
  | { name: "artist"; id: string }
  | { name: "album"; id: string }
  | { name: "playlist"; id: string };

class Nav {
  stack = $state<View[]>([{ name: "search" }]);
  get view(): View {
    return this.stack[this.stack.length - 1];
  }
  go(v: View) {
    const cur = this.view;
    if (cur.name === v.name && (cur as any).id === (v as any).id) return;
    // Sidebar destinations reset history so "back" never gets confusing.
    if (v.name === "search" || v.name === "liked" || v.name === "playlist") this.stack = [v];
    else this.stack = [...this.stack, v];
  }
  back() {
    if (this.stack.length > 1) this.stack = this.stack.slice(0, -1);
  }
}
export const nav = new Nav();

// ---------- Toast ----------

class Toast {
  msg = $state("");
  #timer: ReturnType<typeof setTimeout> | undefined;
  show(e: unknown) {
    this.msg = errMsg(e);
    console.warn("TOAST", this.msg);
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.msg = ""), 5000);
  }
}
export const toast = new Toast();

// ---------- Player ----------
//
// Audio is played inside XSTIFY by Spotify's official Web Playback SDK.
// XSTIFY shows up as a Spotify Connect device; we tell Spotify to play on it.

declare global {
  interface Window {
    onSpotifyWebPlaybackSDKReady?: () => void;
    Spotify: any;
  }
}

const SDK_URL = "https://sdk.scdn.co/spotify-player.js";

function loadSdk(): Promise<void> {
  if (window.Spotify) return Promise.resolve();
  return new Promise((resolve, reject) => {
    window.onSpotifyWebPlaybackSDKReady = () => resolve();
    const s = document.createElement("script");
    s.src = SDK_URL;
    s.onerror = () => reject("Couldn't load Spotify's player. Check your connection.");
    document.head.appendChild(s);
  });
}

const idFromUri = (uri: string) => uri?.split(":").pop() ?? "";

/** Convert the SDK's track object into the Web API shape the UI uses. */
function fromSdk(t: any): Track {
  return {
    id: t.id ?? idFromUri(t.uri),
    uri: t.uri,
    name: t.name,
    duration_ms: t.duration_ms,
    artists: (t.artists ?? []).map((a: any) => ({ id: idFromUri(a.uri), name: a.name })),
    album: { id: idFromUri(t.album?.uri), name: t.album?.name ?? "", images: t.album?.images ?? [] },
  };
}

function loadVolume(): number {
  try {
    const raw = localStorage.getItem("volume");
    const v = Number(raw);
    return raw !== null && Number.isFinite(v) ? v : 70;
  } catch {
    return 70;
  }
}

class Player {
  current = $state<Track | undefined>();
  playing = $state(false);
  loading = $state(false);
  position = $state(0);
  volume = $state(loadVolume());
  hasNext = $state(false);

  #sdk: any;
  #deviceId = "";
  #ready: Promise<void> | undefined;

  /** Load the SDK and register XSTIFY as a playback device. */
  init() {
    this.#ready ??= this.#connect().catch((e) => {
      this.#ready = undefined; // allow a retry on the next play
      throw e;
    });
    return this.#ready;
  }

  async #connect() {
    // Advance the seek bar locally; the SDK only reports position on changes.
    setInterval(() => {
      if (this.playing && this.current) this.position = Math.min(this.position + 500, this.current.duration_ms);
    }, 500);
    this.#registerMediaKeys();
    await loadSdk();
    const sdk = new window.Spotify.Player({
      name: "XSTIFY",
      volume: this.volume / 100,
      getOAuthToken: (cb: (t: string) => void) =>
        invoke<string>("access_token").then(cb).catch((e) => toast.show(e)),
    });
    this.#sdk = sdk;

    const ready = new Promise<void>((resolve) => {
      sdk.addListener("ready", ({ device_id }: { device_id: string }) => {
        this.#deviceId = device_id;
        resolve();
      });
    });
    sdk.addListener("not_ready", () => (this.#deviceId = ""));
    sdk.addListener("player_state_changed", (s: any) => this.#onState(s));
    for (const ev of ["initialization_error", "authentication_error", "account_error", "playback_error"]) {
      sdk.addListener(ev, ({ message }: { message: string }) => {
        this.loading = false;
        toast.show(ev === "account_error" ? "Spotify Premium is required to play music." : message);
      });
    }

    if (!(await sdk.connect())) throw "Couldn't start the player.";
    await ready;
  }

  #onState(s: any) {
    if (!s) {
      this.playing = false; // playback moved to another device
      return;
    }
    const t = s.track_window?.current_track;
    if (t) {
      const changed = this.current?.uri !== t.uri;
      this.current = fromSdk(t);
      if (changed) this.#publishNowPlaying();
    }
    this.position = s.position ?? 0;
    this.playing = !s.paused;
    this.loading = !!s.loading;
    this.hasNext = (s.track_window?.next_tracks?.length ?? 0) > 0;
    if ("mediaSession" in navigator) {
      navigator.mediaSession.playbackState = this.playing ? "playing" : "paused";
      try {
        navigator.mediaSession.setPositionState({
          duration: (this.current?.duration_ms ?? 0) / 1000,
          position: Math.min(this.position, this.current?.duration_ms ?? 0) / 1000,
          playbackRate: 1,
        });
      } catch {}
    }
  }

  /** Tell Windows what's playing: cover art on Stream Deck, media keys, volume flyout. */
  #publishNowPlaying() {
    const t = this.current;
    if (!t || !("mediaSession" in navigator)) return;
    const images = [...t.album.images].sort((a, b) => (a.width ?? 0) - (b.width ?? 0));
    navigator.mediaSession.metadata = new MediaMetadata({
      title: t.name,
      artist: t.artists.map((a) => a.name).join(", "),
      album: t.album.name,
      artwork: images.map((i) => ({ src: i.url, sizes: `${i.width}x${i.height}`, type: "image/jpeg" })),
    });
  }

  #registerMediaKeys() {
    if (!("mediaSession" in navigator)) return;
    const ms = navigator.mediaSession;
    const set = (a: MediaSessionAction, h: MediaSessionActionHandler) => {
      try {
        ms.setActionHandler(a, h);
      } catch {}
    };
    set("play", () => !this.playing && this.toggle());
    set("pause", () => this.playing && this.toggle());
    set("nexttrack", () => this.next());
    set("previoustrack", () => this.prev());
    set("seekto", (d) => d.seekTime != null && this.seek(d.seekTime * 1000));
    set("seekforward", (d) => this.seek(this.position + (d.seekOffset ?? 10) * 1000));
    set("seekbackward", (d) => this.seek(Math.max(0, this.position - (d.seekOffset ?? 10) * 1000)));
  }

  // ----- controls -----

  /** Play `tracks[i]`, queueing the surrounding list so next/previous work. */
  async playList(tracks: Track[], i: number) {
    this.#sdk?.activateElement?.(); // satisfies the webview's autoplay policy (must run on the click)
    this.current = tracks[i];
    this.position = 0;
    this.loading = true;
    try {
      await this.init();
      const start = Math.max(0, i - 20);
      const uris = tracks.slice(start, start + 100).map((t) => t.uri);
      await invoke("api_send", {
        method: "PUT",
        path: `/me/player/play?device_id=${this.#deviceId}`,
        body: { uris, offset: { position: i - start } },
      });
    } catch (e) {
      this.loading = false;
      toast.show(e);
    }
  }

  toggle() {
    if (this.current) this.#sdk?.togglePlay();
  }

  next() {
    this.#sdk?.nextTrack();
  }

  prev() {
    if (this.position > 3000) this.seek(0);
    else this.#sdk?.previousTrack();
  }

  seek(ms: number) {
    this.position = ms;
    this.#sdk?.seek(Math.round(ms));
  }

  setVolume(v: number) {
    this.volume = v;
    try {
      localStorage.setItem("volume", String(v));
    } catch {}
    this.#sdk?.setVolume(v / 100);
  }
}
export const player = new Player();
