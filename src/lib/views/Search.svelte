<script module lang="ts">
  import type { Album, Artist, Track } from "../api";
  // Kept at module level so results survive navigating to an artist and back.
  let query = $state("");
  let results = $state<{ tracks: Track[]; artists: Artist[]; albums: Album[] } | null>(null);
</script>

<script lang="ts">
  import { get, img } from "../api";
  import { nav, toast } from "../state.svelte";
  import TrackList from "../TrackList.svelte";
  import Card from "../Card.svelte";
  import Icon from "../Icon.svelte";

  let { searchEl = $bindable() }: { searchEl?: HTMLInputElement } = $props();
  let loading = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  let seq = 0;

  function onInput() {
    clearTimeout(timer);
    timer = setTimeout(run, 300);
  }

  async function run() {
    const q = query.trim();
    if (!q) {
      results = null;
      return;
    }
    const mine = ++seq;
    loading = true;
    try {
      const r = await get(`/search?q=${encodeURIComponent(q)}&type=track,artist,album&limit=10&market=from_token`);
      if (mine !== seq) return; // a newer search started
      results = {
        tracks: (r.tracks?.items ?? []).filter(Boolean) as Track[],
        artists: (r.artists?.items ?? []).filter(Boolean) as Artist[],
        albums: (r.albums?.items ?? []).filter(Boolean) as Album[],
      };
    } catch (e) {
      toast.show(e);
    } finally {
      if (mine === seq) loading = false;
    }
  }
</script>

<div class="bar">
  <Icon name="search" />
  <!-- svelte-ignore a11y_autofocus -->
  <input
    bind:this={searchEl}
    bind:value={query}
    oninput={onInput}
    onkeydown={(e) => e.key === "Enter" && (clearTimeout(timer), run())}
    placeholder="Search songs, artists, albums"
    spellcheck="false"
    autofocus
  />
  {#if loading}<span class="spin"></span>{/if}
</div>

{#if !results}
  <div class="empty">
    <Icon name="search" size={48} />
    <p>Find any song, artist or album.</p>
    <p class="hint"><kbd>Ctrl</kbd> + <kbd>K</kbd> jumps here from anywhere.</p>
  </div>
{:else if !results.tracks.length && !results.artists.length && !results.albums.length}
  <div class="empty"><p>No results for "{query}".</p></div>
{:else}
  {#if results.artists.length}
    {@const top = results.artists[0]}
    <section class="top">
      <div>
        <h2>Top result</h2>
        <button class="topcard" onclick={() => nav.go({ name: "artist", id: top.id })}>
          <img src={img(top.images, 120)} alt="" />
          <div class="tn">{top.name}</div>
          <div class="ts">Artist</div>
        </button>
      </div>
      {#if results.tracks.length}
        <div class="songs">
          <h2>Songs</h2>
          <TrackList tracks={results.tracks.slice(0, 5)} album={false} />
        </div>
      {/if}
    </section>
  {:else if results.tracks.length}
    <h2>Songs</h2>
    <TrackList tracks={results.tracks.slice(0, 5)} />
  {/if}

  {#if results.artists.length}
    <h2>Artists</h2>
    <div class="grid">
      {#each results.artists as a (a.id)}
        <Card image={img(a.images, 160)} title={a.name} subtitle="Artist" round onclick={() => nav.go({ name: "artist", id: a.id })} />
      {/each}
    </div>
  {/if}

  {#if results.albums.length}
    <h2>Albums</h2>
    <div class="grid">
      {#each results.albums as al (al.id)}
        <Card image={img(al.images, 160)} title={al.name} subtitle={`${al.release_date?.slice(0, 4) ?? ""} · ${al.artists.map((a) => a.name).join(", ")}`} onclick={() => nav.go({ name: "album", id: al.id })} />
      {/each}
    </div>
  {/if}

  {#if results.tracks.length > 5}
    <h2>More songs</h2>
    <TrackList tracks={results.tracks} />
  {/if}
{/if}

<style>
  .bar {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface-2);
    border-radius: 999px;
    padding: 0 18px;
    height: 48px;
    max-width: 520px;
    color: var(--muted);
    margin-bottom: 8px;
    box-shadow: 0 0 0 12px var(--bg);
  }
  .bar:focus-within { outline: 2px solid var(--text); }
  .bar input { flex: 1; background: none; border: 0; outline: 0; color: var(--text); font: inherit; font-size: 15px; }
  .spin { width: 16px; height: 16px; border: 2px solid var(--muted); border-top-color: transparent; border-radius: 50%; animation: s 0.7s linear infinite; }
  @keyframes s { to { transform: rotate(360deg); } }
  .empty { display: grid; place-items: center; text-align: center; color: var(--muted); padding: 80px 0; gap: 6px; }
  .hint { font-size: 13px; }
  kbd { background: var(--surface-2); border-radius: 4px; padding: 1px 6px; font: inherit; font-size: 12px; }
  .top { display: grid; grid-template-columns: minmax(220px, 2fr) 3fr; gap: 24px; }
  .topcard { display: block; width: 100%; text-align: left; background: var(--surface); border-radius: 10px; padding: 20px; }
  .topcard:hover { background: var(--hover); }
  .topcard img { width: 96px; height: 96px; border-radius: 50%; object-fit: cover; background: var(--surface-2); }
  .tn { font-size: 28px; font-weight: 700; margin-top: 14px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ts { color: var(--muted); font-size: 13px; margin-top: 4px; }
  @media (max-width: 860px) { .top { grid-template-columns: 1fr; } }
</style>
