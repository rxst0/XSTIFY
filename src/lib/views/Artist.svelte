<script lang="ts">
  import { get, img, type Album, type Artist, type Track } from "../api";
  import { nav, player, toast } from "../state.svelte";
  import TrackList from "../TrackList.svelte";
  import Card from "../Card.svelte";
  import Header from "../Header.svelte";

  let { id }: { id: string } = $props();
  let artist = $state<Artist | null>(null);
  let top = $state<Track[]>([]);
  let albums = $state<Album[]>([]);
  let showAll = $state(false);

  $effect(() => {
    const cur = id;
    artist = null;
    top = [];
    albums = [];
    showAll = false;
    Promise.allSettled([
      get<Artist>(`/artists/${cur}`),
      get(`/artists/${cur}/top-tracks?market=from_token`),
      get(`/artists/${cur}/albums?include_groups=album,single&market=from_token`),
    ]).then(([a, t, al]) => {
      if (cur !== id) return;
      if (a.status === "fulfilled") artist = a.value;
      else toast.show(a.reason);
      if (t.status === "fulfilled") top = t.value.tracks ?? [];
      if (al.status === "fulfilled") albums = al.value.items ?? [];
    });
  });
</script>

{#if artist}
  <Header
    image={img(artist.images, 240)}
    round
    kind="Artist"
    title={artist.name}
    meta={artist.followers ? `${artist.followers.total.toLocaleString()} followers` : ""}
    onplay={top.length ? () => player.playList(top, 0) : undefined}
  />

  {#if top.length}
    <h2>Popular</h2>
    <TrackList tracks={showAll ? top : top.slice(0, 5)} numbered album={false} />
    {#if top.length > 5}
      <button class="more" onclick={() => (showAll = !showAll)}>{showAll ? "Show less" : "Show more"}</button>
    {/if}
  {/if}

  {#if albums.length}
    <h2>Discography</h2>
    <div class="grid">
      {#each albums as al (al.id)}
        <Card image={img(al.images, 160)} title={al.name} subtitle={`${al.release_date?.slice(0, 4) ?? ""} · ${al.album_type === "single" ? "Single" : "Album"}`} onclick={() => nav.go({ name: "album", id: al.id })} />
      {/each}
    </div>
  {/if}
{:else}
  <div class="loading">Loading…</div>
{/if}

<style>
  .more { color: var(--accent); font-weight: 600; font-size: 13px; padding: 8px 12px; }
  .more:hover { filter: brightness(1.2); }
  .loading { color: var(--muted); padding: 40px 0; }
</style>
