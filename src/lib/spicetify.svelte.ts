// Spicetify compatibility layer.
//
// Spicetify extensions are written against a global `Spicetify` object that normally only
// exists inside a patched official Spotify app. This file provides XSTIFY's own
// implementation of the commonly used parts of that API, mapped onto XSTIFY's player and UI,
// so many simple extensions can run unmodified. Not provided: Spotify's private services
// (Platform internals, CosmosAsync, GraphQL) and the official app's own page layout.
//
// Written from scratch for XSTIFY; API names follow Spicetify's public docs for compatibility.
import { invoke } from "@tauri-apps/api/core";
import { errMsg, get, type Track } from "./api";
import { nav, player, toast } from "./state.svelte";

// ---------------------------------------------------------------- small helpers

const ICON_PATHS: Record<string, string> = {
  check: '<path d="M13.3 3.3 6 10.6 2.7 7.3 1.3 8.7 6 13.4l8.7-8.7z"/>',
  x: '<path d="M3.5 2.1 8 6.6l4.5-4.5 1.4 1.4L9.4 8l4.5 4.5-1.4 1.4L8 9.4l-4.5 4.5-1.4-1.4L6.6 8 2.1 3.5z"/>',
  heart: '<path d="M8 14s-5.5-3.3-6.6-6.7C.6 4.8 2.3 2.5 4.7 2.5c1.4 0 2.4.8 3.3 1.9.9-1.1 1.9-1.9 3.3-1.9 2.4 0 4.1 2.3 3.3 4.8C13.5 10.7 8 14 8 14z" fill="none" stroke="currentColor" stroke-width="1.3"/>',
  "heart-active": '<path d="M8 14s-5.5-3.3-6.6-6.7C.6 4.8 2.3 2.5 4.7 2.5c1.4 0 2.4.8 3.3 1.9.9-1.1 1.9-1.9 3.3-1.9 2.4 0 4.1 2.3 3.3 4.8C13.5 10.7 8 14 8 14z"/>',
  play: '<path d="M4 2.5v11l9-5.5z"/>',
  pause: '<path d="M3.5 2.5h3v11h-3zM9.5 2.5h3v11h-3z"/>',
  "skip-forward": '<path d="M3 2.5v11l7-5.5zM11.5 2.5h1.7v11h-1.7z"/>',
  "skip-back": '<path d="M13 2.5v11l-7-5.5zM2.8 2.5h1.7v11H2.8z"/>',
  plus2px: '<path d="M7.2 2h1.6v5.2H14v1.6H8.8V14H7.2V8.8H2V7.2h5.2z"/>',
  minus: '<path d="M2 7.2h12v1.6H2z"/>',
  search: '<path d="M7 1.5a5.5 5.5 0 0 1 4.4 8.8l3.2 3.2-1.1 1.1-3.2-3.2A5.5 5.5 0 1 1 7 1.5zm0 1.6a3.9 3.9 0 1 0 0 7.8 3.9 3.9 0 0 0 0-7.8z"/>',
  more: '<path d="M2 6.5h3v3H2zM6.5 6.5h3v3h-3zM11 6.5h3v3h-3z"/>',
};

/** Accepts a full <svg>, an SVGIcons name, or raw path markup. */
function iconHtml(icon: unknown): string {
  if (!icon || typeof icon !== "string") return "";
  if (icon.trim().startsWith("<svg")) return icon;
  const inner = ICON_PATHS[icon] ?? (icon.trim().startsWith("<") ? icon : "");
  return inner ? `<svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor" aria-hidden="true">${inner}</svg>` : "";
}

const TYPE = {
  TRACK: "track",
  ARTIST: "artist",
  ALBUM: "album",
  PLAYLIST: "playlist",
  PLAYLIST_V2: "playlist-v2",
  EPISODE: "episode",
  SHOW: "show",
  USER: "user",
  LOCAL: "local",
  COLLECTION: "collection",
  APPLICATION: "application",
} as const;

class URI {
  static Type = TYPE;
  constructor(
    public type: string,
    public id: string,
    private raw: string,
  ) {}
  static fromString(s: string): URI {
    const str = String(s ?? "");
    if (str.startsWith("local:")) return new URI(TYPE.LOCAL, str.slice(6), str);
    const parts = str.split(":");
    if (parts[0] !== "spotify" || parts.length < 3) throw new TypeError(`Invalid URI: ${str}`);
    return new URI(parts[1], parts.slice(2).join(":"), str);
  }
  static from(s: string) {
    try {
      return URI.fromString(s);
    } catch {
      return null;
    }
  }
  static isTrack = (u: string) => URI.from(u)?.type === TYPE.TRACK;
  static isArtist = (u: string) => URI.from(u)?.type === TYPE.ARTIST;
  static isAlbum = (u: string) => URI.from(u)?.type === TYPE.ALBUM;
  static isPlaylistV1OrV2 = (u: string) => URI.from(u)?.type === TYPE.PLAYLIST;
  static isEpisode = (u: string) => URI.from(u)?.type === TYPE.EPISODE;
  static isLocalTrack = (u: string) => URI.from(u)?.type === TYPE.LOCAL;
  toURI() {
    return this.raw;
  }
  toString() {
    return this.raw;
  }
}

// ---------------------------------------------------------------- UI registries (rendered by XSTIFY)

type Slot = "playbar" | "widgets" | "topbar";
const slots: Record<Slot, HTMLElement | null> = { playbar: null, widgets: null, topbar: null };
const mounted: Record<Slot, Set<HTMLElement>> = { playbar: new Set(), widgets: new Set(), topbar: new Set() };

/** Called by XSTIFY components to provide a place for extension buttons. */
export function attachSlot(slot: Slot, el: HTMLElement | null) {
  slots[slot] = el;
  if (el) for (const b of mounted[slot]) el.append(b);
}

class ButtonBase {
  element: HTMLButtonElement;
  #label = "";
  #icon = "";
  #active = false;
  #disabled = false;
  constructor(
    private slot: Slot,
    label: string,
    icon: string,
    onClick: (self: any) => void,
    disabled = false,
    active = false,
    registerOnCreate = true,
  ) {
    this.element = document.createElement("button");
    this.element.className = "xs-ext-btn";
    this.element.addEventListener("click", (e) => {
      e.stopPropagation();
      if (!this.#disabled) onClick?.(this);
    });
    this.label = label;
    this.icon = icon;
    this.active = active;
    this.disabled = disabled;
    if (registerOnCreate) this.register();
  }
  get label() {
    return this.#label;
  }
  set label(v: string) {
    this.#label = v;
    this.element.title = v;
    this.element.setAttribute("aria-label", v);
  }
  get icon() {
    return this.#icon;
  }
  set icon(v: string) {
    this.#icon = v;
    this.element.innerHTML = iconHtml(v) || `<span>${String(v ?? "").slice(0, 2)}</span>`;
  }
  get active() {
    return this.#active;
  }
  set active(v: boolean) {
    this.#active = !!v;
    this.element.classList.toggle("active", this.#active);
  }
  get disabled() {
    return this.#disabled;
  }
  set disabled(v: boolean) {
    this.#disabled = !!v;
    this.element.disabled = this.#disabled;
  }
  register() {
    mounted[this.slot].add(this.element);
    slots[this.slot]?.append(this.element);
  }
  deregister() {
    mounted[this.slot].delete(this.element);
    this.element.remove();
  }
}

class PlaybarButton extends ButtonBase {
  constructor(label: string, icon: string, onClick: (s: any) => void, disabled?: boolean, active?: boolean, reg?: boolean) {
    super("playbar", label, icon, onClick, disabled, active, reg);
  }
}
class PlaybarWidget extends ButtonBase {
  constructor(label: string, icon: string, onClick: (s: any) => void, disabled?: boolean, active?: boolean, reg?: boolean) {
    super("widgets", label, icon, onClick, disabled, active, reg);
  }
}
class TopbarButton extends ButtonBase {
  constructor(label: string, icon: string, onClick: (s: any) => void, disabled?: boolean) {
    super("topbar", label, icon, onClick, disabled, false, true);
  }
}

// Menu items appear in Settings → Extensions.
export class MenuItem {
  name = $state("");
  isEnabled = $state<boolean | undefined>(undefined);
  icon: string;
  constructor(
    name: string,
    isEnabled: boolean | undefined,
    public onClick: (self: MenuItem) => void,
    icon = "",
  ) {
    this.name = name;
    this.isEnabled = isEnabled;
    this.icon = icon;
  }
  setState(v: boolean) {
    this.isEnabled = v;
  }
  setName(v: string) {
    this.name = v;
  }
  register() {
    if (!ext.menu.includes(this)) ext.menu.push(this);
  }
  deregister() {
    ext.menu = ext.menu.filter((m) => m !== this);
  }
}
class MenuSubMenu {
  constructor(
    public name: string,
    public items: MenuItem[],
  ) {}
  register() {
    for (const i of this.items) {
      i.name = `${this.name}: ${i.name}`;
      i.register();
    }
  }
  deregister() {
    for (const i of this.items) i.deregister();
  }
}

// Context-menu items appear when right-clicking a song or artist.
export class ContextItem {
  constructor(
    public name: string,
    public onClick: (uris: string[], uids?: string[], contextUri?: string) => void,
    public shouldAdd: (uris: string[], uids?: string[], contextUri?: string) => boolean = () => true,
    public icon = "",
    public disabled = false,
  ) {}
  register() {
    if (!ext.context.includes(this)) ext.context.push(this);
  }
  deregister() {
    ext.context = ext.context.filter((c) => c !== this);
  }
}
class ContextSubMenu {
  constructor(
    public name: string,
    public items: ContextItem[],
    public shouldAdd: (uris: string[]) => boolean = () => true,
  ) {}
  register() {
    for (const i of this.items) {
      const inner = i.shouldAdd;
      i.name = `${this.name}: ${i.name}`;
      i.shouldAdd = (u, a, b) => this.shouldAdd(u) && inner(u, a, b);
      i.register();
    }
  }
  deregister() {
    for (const i of this.items) i.deregister();
  }
}

// ---------------------------------------------------------------- shared state for XSTIFY's UI

class ExtState {
  menu = $state<MenuItem[]>([]);
  context = $state<ContextItem[]>([]);
  modal = $state<{ title: string; content: HTMLElement | string; isLarge?: boolean } | null>(null);
  contextMenu = $state<{ x: number; y: number; uris: string[]; items: ContextItem[] } | null>(null);

  /** Right-click on a song/artist: show extension menu items that apply. */
  openContextMenu(e: MouseEvent, uris: string[]) {
    const items = this.context.filter((c) => {
      try {
        return c.shouldAdd(uris, [], "");
      } catch {
        return false;
      }
    });
    if (!items.length) return;
    e.preventDefault();
    this.contextMenu = { x: e.clientX, y: e.clientY, uris, items };
  }
}
export const ext = new ExtState();

// ---------------------------------------------------------------- Player

type Listener = (e: { type: string; data: any }) => void;
const listeners = new Map<string, Set<Listener>>();
const emit = (type: string, data: any) => listeners.get(type)?.forEach((cb) => {
  try {
    cb({ type, data });
  } catch (err) {
    console.warn("[extension]", err);
  }
});

function itemFor(t: Track) {
  const known = player.known.get(t.uri);
  const explicit = t.explicit ?? known?.explicit ?? false;
  const image = t.album.images?.[0]?.url ?? "";
  const artistUri = (id: string) => (id ? `spotify:artist:${id}` : "");
  return {
    uri: t.uri,
    uid: t.uri,
    name: t.name,
    type: t.uri.startsWith("local:") ? "local" : "track",
    duration: { milliseconds: t.duration_ms },
    isExplicit: explicit,
    isLocal: t.uri.startsWith("local:"),
    artists: t.artists.map((a) => ({ name: a.name, uri: artistUri(a.id), type: "artist" })),
    album: { name: t.album.name, uri: t.album.id ? `spotify:album:${t.album.id}` : "", images: t.album.images ?? [] },
    images: t.album.images ?? [],
    metadata: {
      title: t.name,
      artist_name: t.artists.map((a) => a.name).join(", "),
      artist_uri: artistUri(t.artists[0]?.id ?? ""),
      album_title: t.album.name,
      album_uri: t.album.id ? `spotify:album:${t.album.id}` : "",
      image_url: image,
      image_large_url: image,
      is_explicit: explicit ? "true" : "false",
      "media.type": "audio",
      is_advertisement: "false",
      duration: String(t.duration_ms),
    },
  };
}

const Player = {
  get data() {
    const t = player.current;
    if (!t) return null;
    const item = itemFor(t);
    return {
      item,
      track: item,
      isPaused: !player.playing,
      isBuffering: player.loading,
      duration: t.duration_ms,
      positionAsOfTimestamp: player.position,
      timestamp: Date.now(),
      context: { uri: "", metadata: {} },
      repeat: 0,
      shuffle: false,
    };
  },
  addEventListener(type: string, cb: Listener) {
    if (!listeners.has(type)) listeners.set(type, new Set());
    listeners.get(type)!.add(cb);
  },
  removeEventListener(type: string, cb: Listener) {
    listeners.get(type)?.delete(cb);
  },
  play: () => !player.playing && player.toggle(),
  pause: () => player.playing && player.toggle(),
  togglePlay: () => player.toggle(),
  isPlaying: () => player.playing,
  next: () => player.next(),
  back: () => player.prev(),
  getProgress: () => player.position,
  getDuration: () => player.current?.duration_ms ?? 0,
  getProgressPercent() {
    const d = Player.getDuration();
    return d ? Math.min(1, player.position / d) : 0;
  },
  /** Spicetify accepts either milliseconds or a 0–1 fraction. */
  seek(p: number) {
    const d = Player.getDuration();
    player.seek(p >= 0 && p <= 1 ? p * d : p);
  },
  skipForward: (ms = 15000) => player.seek(player.position + ms),
  skipBack: (ms = 15000) => player.seek(Math.max(0, player.position - ms)),
  getVolume: () => player.volume / 100,
  setVolume: (v: number) => player.setVolume(Math.round(Math.min(1, Math.max(0, v)) * 100)),
  increaseVolume: () => player.setVolume(Math.min(100, player.volume + 10)),
  decreaseVolume: () => player.setVolume(Math.max(0, player.volume - 10)),
  getMute: () => player.volume === 0,
  setMute: (m: boolean) => player.setVolume(m ? 0 : 70),
  toggleMute: () => player.setVolume(player.volume ? 0 : 70),
  getRepeat: () => 0,
  setRepeat: () => {},
  toggleRepeat: () => {},
  getShuffle: () => false,
  setShuffle: () => {},
  toggleShuffle: () => {},
  getHeart: () => false,
  toggleHeart: () => toast.show("Liking songs isn't supported in XSTIFY yet."),
  async playUri(uri: string) {
    const u = URI.from(uri);
    if (u?.type !== TYPE.TRACK) return toast.show("Extensions can only play single songs in XSTIFY.");
    const t = await get<Track>(`/tracks/${u.id}`);
    player.playList([t], 0);
  },
};

// Fire Spicetify's player events from XSTIFY's player state.
let watching = false;
function watchPlayer() {
  if (watching) return;
  watching = true;
  $effect.root(() => {
    let lastUri: string | undefined;
    let lastPlaying: boolean | undefined;
    $effect(() => {
      const uri = player.current?.uri;
      if (uri && uri !== lastUri) {
        lastUri = uri;
        queueMicrotask(() => emit("songchange", Player.data));
      }
    });
    $effect(() => {
      const p = player.playing;
      if (lastPlaying !== undefined && p !== lastPlaying) emit("onplaypause", Player.data);
      lastPlaying = p;
    });
    $effect(() => {
      emit("onprogress", player.position);
    });
    let firstNav = true;
    $effect(() => {
      const loc = { pathname: pathFor(nav.view) };
      if (firstNav) return void (firstNav = false);
      queueMicrotask(() => historyListeners.forEach((cb) => {
        try {
          cb(loc);
        } catch (err) {
          console.warn("[extension]", err);
        }
      }));
    });
  });
}

// ---------------------------------------------------------------- CosmosAsync (web requests)

const SPOTIFY_API = "https://api.spotify.com/v1";

/**
 * Spicetify's CosmosAsync sends requests with the user's Spotify login. Here, Spotify Web API
 * calls go through XSTIFY's own signed-in connection (the token never reaches extensions);
 * other https addresses are fetched normally. Spotify-internal addresses (sp://, wg://)
 * only exist in the official app and are rejected.
 */
async function cosmos(method: "GET" | "POST" | "PUT" | "DELETE", url: string, body?: unknown): Promise<any> {
  url = String(url);
  if (url.startsWith(SPOTIFY_API)) {
    const call = (path: string) =>
      method === "GET" ? invoke("api_get", { path }) : invoke("api_send", { method, path, body: body ?? null });
    const path = url.slice(SPOTIFY_API.length);
    try {
      return await call(path);
    } catch (e) {
      // Newer Spotify apps must use /playlists/{id}/items instead of /tracks.
      const m = path.match(/^\/playlists\/([^/?]+)\/tracks(.*)$/);
      if (m && /\b(403|404)\b/.test(String(e))) {
        const r: any = await call(`/playlists/${m[1]}/items${m[2]}`);
        if (Array.isArray(r?.items)) r.items = r.items.map((i: any) => ({ ...i, track: i?.track ?? i?.item }));
        return r;
      }
      throw new Error(errMsg(e));
    }
  }
  if (url.startsWith("https://")) {
    // Wikipedia's API only allows apps like this to call it with origin=*.
    if (/^https:\/\/[a-z.]*wiki[mp]edia\.org\/w\/api\.php/.test(url) && !/[?&]origin=/.test(url))
      url += (url.includes("?") ? "&" : "?") + "origin=*";
    const r = await fetch(url, {
      method,
      headers: body ? { "content-type": "application/json" } : undefined,
      body: body ? JSON.stringify(body) : undefined,
    });
    const text = await r.text();
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  throw new Error(`"${url}" is only available inside the official Spotify app.`);
}

const CosmosAsync = {
  get: (url: string, _body?: unknown) => cosmos("GET", url),
  post: (url: string, body?: unknown) => cosmos("POST", url, body),
  put: (url: string, body?: unknown) => cosmos("PUT", url, body),
  del: (url: string, body?: unknown) => cosmos("DELETE", url, body),
  delete: (url: string, body?: unknown) => cosmos("DELETE", url, body),
};

/** Add songs to a playlist (Spicetify.Platform.PlaylistAPI.add). */
async function playlistAdd(playlist: string, uris: string[], opts?: { before?: number | string; after?: number | string }) {
  const id = URI.from(playlist)?.id ?? String(playlist).split(":").pop();
  const body: Record<string, unknown> = { uris: uris.filter(Boolean) };
  if (opts?.before === 0 || opts?.before === "start") body.position = 0;
  return cosmos("POST", `${SPOTIFY_API}/playlists/${id}/tracks`, body);
}

// Page changes, for Spicetify.Platform.History.listen
const historyListeners = new Set<(loc: { pathname: string }) => void>();
const pathFor = (v: typeof nav.view) => ("id" in v ? `/${v.name}/${v.id}` : `/${v.name}`);

// ---------------------------------------------------------------- the Spicetify object

function buildSpicetify() {
  const LocalStorage = {
    get: (k: string) => {
      try {
        return localStorage.getItem(`spice:${k}`);
      } catch {
        return null;
      }
    },
    set: (k: string, v: string) => {
      try {
        localStorage.setItem(`spice:${k}`, String(v));
      } catch {}
    },
    remove: (k: string) => {
      try {
        localStorage.removeItem(`spice:${k}`);
      } catch {}
    },
    clear: () => {
      try {
        Object.keys(localStorage)
          .filter((k) => k.startsWith("spice:"))
          .forEach((k) => localStorage.removeItem(k));
      } catch {}
    },
  };

  const ready = { on: (cb: () => void) => queueMicrotask(cb) };

  return {
    xstify: true, // lets extensions detect XSTIFY
    Player,
    get Queue() {
      const d = Player.data;
      return d ? { ...d, track: d.item, nextTracks: [], prevTracks: [] } : null;
    },
    URI,
    LocalStorage,
    SVGIcons: ICON_PATHS,
    Locale: { getLocale: () => navigator.language || "en", get: (k: string) => k },
    Config: { version: "xstify", current_theme: "XSTIFY", color_scheme: "", extensions: [], custom_apps: [] },
    Events: { webpackLoaded: ready, platformLoaded: ready },
    showNotification(msg: unknown, _isError?: boolean) {
      toast.show(typeof msg === "string" ? msg : String((msg as any)?.textContent ?? msg ?? ""));
    },
    Menu: { Item: MenuItem, SubMenu: MenuSubMenu },
    ContextMenu: { Item: ContextItem, SubMenu: ContextSubMenu },
    Playbar: { Button: PlaybarButton, Widget: PlaybarWidget },
    Topbar: { Button: TopbarButton },
    PopupModal: {
      display(o: { title?: string; content: HTMLElement | string; isLarge?: boolean }) {
        ext.modal = { title: o.title ?? "", content: o.content, isLarge: o.isLarge };
      },
      hide() {
        ext.modal = null;
      },
    },
    CosmosAsync,
    Platform: {
      ClipboardAPI: { copy: (t: string) => navigator.clipboard.writeText(String(t)), paste: () => navigator.clipboard.readText() },
      PlaylistAPI: { add: playlistAdd },
      History: {
        get location() {
          return { pathname: pathFor(nav.view) };
        },
        push(p: string | { pathname: string }) {
          const path = typeof p === "string" ? p : p?.pathname;
          const [, kind, id] = String(path).split("/");
          if ((kind === "artist" || kind === "album" || kind === "playlist") && id) nav.go({ name: kind, id } as any);
        },
        listen(cb: (loc: { pathname: string }) => void) {
          historyListeners.add(cb);
          return () => historyListeners.delete(cb);
        },
      },
    },
  };
}

// ---------------------------------------------------------------- loading extensions

export type ExtInfo = { name: string; enabled: boolean };

let loaded = false;

/** Install `window.Spicetify` and run all enabled extensions (once per app start). */
export async function loadExtensions() {
  if (loaded) return;
  loaded = true;
  (window as any).Spicetify = buildSpicetify();
  watchPlayer();
  let list: ExtInfo[] = [];
  try {
    list = await invoke<ExtInfo[]>("ext_list");
  } catch {
    return;
  }
  for (const e of list.filter((x) => x.enabled)) {
    try {
      const code = await invoke<string>("ext_code", { name: e.name });
      // Indirect eval runs the extension in global scope, like Spotify's client does.
      (0, eval)(`${code}\n//# sourceURL=xstify-extension/${encodeURIComponent(e.name)}`);
    } catch (err) {
      toast.show(`Extension "${e.name}" failed to start: ${err}`);
    }
  }
}
