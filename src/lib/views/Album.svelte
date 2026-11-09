<script lang="ts">
  import { fmt, get, img, type Track } from "../api";
  import { nav, player, toast } from "../state.svelte";
  import TrackList from "../TrackList.svelte";
  import Header from "../Header.svelte";

  let { id }: { id: string } = $props();
  let album = $state<any>(null);
  let tracks = $state<Track[]>([]);

  $effect(() => {
    const cur = id;
    album = null;
    tracks = [];
    get(`/albums/${cur}?market=from_token`)
      .then((a) => {
        if (cur !== id) return;
        album = a;
        // Album track objects don't include the album, so attach it for the player bar.
        const ref = { id: a.id, name: a.name, images: a.images };
        tracks = (a.tracks?.items ?? []).map((t: any) => ({ ...t, album: ref }));
      })
      .catch((e) => toast.show(e));
  });

  let total = $derived(tracks.reduce((s, t) => s + t.duration_ms, 0));
</script>

{#if album}
  <Header
    image={img(album.images, 240)}
    kind={album.album_type === "single" ? "Single" : "Album"}
    title={album.name}
    meta={` · ${album.release_date?.slice(0, 4) ?? ""} · ${tracks.length} songs, ${fmt(total)}`}
    onplay={tracks.length ? () => player.playList(tracks, 0) : undefined}
  >
    {#each album.artists as a, j}<button class="link artist" onclick={() => nav.go({ name: "artist", id: a.id })}>{a.name}</button>{#if j < album.artists.length - 1}, {/if}{/each}
  </Header>
  <TrackList {tracks} covers={false} album={false} numbered />
{:else}
  <div class="loading">Loading…</div>
{/if}

<style>
  .artist { color: var(--text); font-weight: 600; }
  .loading { color: var(--muted); padding: 40px 0; }
</style>
