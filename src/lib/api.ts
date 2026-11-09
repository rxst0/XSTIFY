import { invoke } from "@tauri-apps/api/core";

export type Image = { url: string; width: number | null; height: number | null };
export type ArtistRef = { id: string; name: string };
export type AlbumRef = { id: string; name: string; images: Image[] };
export type Track = {
  id: string;
  uri: string;
  name: string;
  duration_ms: number;
  explicit?: boolean;
  artists: ArtistRef[];
  album: AlbumRef;
};
export type Artist = ArtistRef & { images: Image[]; followers?: { total: number }; genres?: string[] };
export type Album = AlbumRef & {
  artists: ArtistRef[];
  release_date: string;
  album_type: string;
  total_tracks: number;
};
export type Playlist = { id: string; name: string; images: Image[] | null; owner: { display_name: string } };
export type Page<T> = { items: T[]; next: string | null; total: number };

export function get<T = any>(path: string): Promise<T> {
  return invoke<T>("api_get", { path });
}

/** Smallest image that's at least `size` px wide (keeps memory down). */
export function img(images: Image[] | null | undefined, size = 64): string {
  if (!images?.length) return "";
  const sorted = [...images].sort((a, b) => (a.width ?? 0) - (b.width ?? 0));
  return (sorted.find((i) => (i.width ?? 0) >= size) ?? sorted[sorted.length - 1]).url;
}

export function fmt(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

export function artistNames(t: { artists: ArtistRef[] }): string {
  return t.artists.map((a) => a.name).join(", ");
}

/** Playlist / liked-songs items wrap the track; skip local files, episodes and removed tracks. */
export function unwrapTracks(items: any[]): Track[] {
  return items
    .map((it) => it?.track ?? it?.item)
    .filter((t) => t && t.type === "track" && t.id && !t.is_local);
}

export function errMsg(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Something went wrong";
}
