<script lang="ts">
  // Your own music files, played by XSTIFY (not through Spotify).
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { Track } from "../api";
  import { player, toast } from "../state.svelte";
  import TrackList from "../TrackList.svelte";
  import Header from "../Header.svelte";

  type LocalTrack = { id: string; path: string; title: string; artist: string };
  const EXTS = ["mp3", "m4a", "aac", "flac", "wav", "ogg", "opus", "webm", "wma"];

  let tracks = $state<Track[]>([]);
  let dragging = $state(false);

  const toTrack = (t: LocalTrack): Track => ({
    id: `local-${t.id}`,
    uri: `local:${t.path}`,
    name: t.title,
    duration_ms: 0,
    artists: [{ id: "", name: t.artist }],
    album: { id: "", name: "Local Files", images: [] },
  });

  const set = (list: LocalTrack[]) => (tracks = list.map(toTrack));

  $effect(() => {
    invoke<LocalTrack[]>("local_list").then(set).catch((e) => toast.show(e));
    // Drag music files or folders onto the window to add them.
    const un = getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") dragging = true;
      else if (p.type === "leave") dragging = false;
      else if (p.type === "drop") {
        dragging = false;
        add(p.paths);
      }
    });
    return () => {
      un.then((f) => f());
    };
  });

  async function add(paths: string[] | string | null) {
    if (!paths || (Array.isArray(paths) && !paths.length)) return;
    const before = tracks.length;
    try {
      set(await invoke<LocalTrack[]>("local_add", { paths: Array.isArray(paths) ? paths : [paths] }));
      const added = tracks.length - before;
      toast.show(added ? `Added ${added} song${added === 1 ? "" : "s"}` : "No new music files found there.");
    } catch (e) {
      toast.show(e);
    }
  }

  const addFiles = async () =>
    add(await open({ multiple: true, title: "Add music files", filters: [{ name: "Audio", extensions: EXTS }] }));
  const addFolder = async () => add(await open({ directory: true, multiple: true, title: "Add a music folder" }));

  async function remove(t: Track) {
    try {
      set(await invoke<LocalTrack[]>("local_remove", { id: t.id.slice("local-".length) }));
    } catch (e) {
      toast.show(e);
    }
  }
</script>

<Header
  image=""
  kind="Playlist"
  title="Local Files"
  meta={`${tracks.length} song${tracks.length === 1 ? "" : "s"} on this PC`}
  onplay={tracks.length ? () => player.playList(tracks, 0) : undefined}
/>

<div class="actions">
  <button class="primary" onclick={addFiles}>Add files</button>
  <button onclick={addFolder}>Add folder</button>
  <span class="hint">or drag songs and folders onto this window</span>
</div>

{#if tracks.length}
  <TrackList {tracks} covers={false} album={false} onremove={remove} />
{:else}
  <div class="empty" class:dragging>
    <p><b>Add your own music</b></p>
    <p>MP3, M4A, FLAC, WAV, OGG and more. Files stay where they are; XSTIFY just remembers where to find them.</p>
    <p class="small">Name files "Artist - Title" to show the artist.</p>
  </div>
{/if}

{#if dragging && tracks.length}<div class="drop">Drop to add to Local Files</div>{/if}

<style>
  .actions { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin: 0 0 16px; }
  .actions button { padding: 8px 16px; border-radius: 999px; font-weight: 600; border: 1px solid var(--accent); color: var(--accent); }
  .actions button:hover { background: color-mix(in srgb, var(--accent) 15%, transparent); }
  .actions .primary { background: var(--accent); color: var(--on-accent); }
  .actions .primary:hover { background: var(--accent); filter: brightness(1.1); }
  .hint { color: var(--muted); font-size: 13px; }
  .empty { border: 2px dashed var(--surface-3); border-radius: 12px; padding: 32px; text-align: center; color: var(--muted); display: grid; gap: 6px; }
  .empty.dragging { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 8%, transparent); }
  .empty p { margin: 0; }
  .empty b { color: var(--text); }
  .small { font-size: 12px; }
  .drop { position: fixed; inset: 44px 8px 92px 276px; border: 2px dashed var(--accent); border-radius: 12px; background: color-mix(in srgb, var(--bg) 70%, transparent); display: grid; place-items: center; color: var(--accent); font-weight: 700; pointer-events: none; z-index: 5; }
</style>
