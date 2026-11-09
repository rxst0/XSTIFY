<script lang="ts">
  // Liked Songs or a playlist: paged track list that loads more as you scroll.
  import { errMsg, get, img, unwrapTracks, type Track } from "../api";
  import { player, toast } from "../state.svelte";
  import TrackList from "../TrackList.svelte";
  import Header from "../Header.svelte";
  import LoadMore from "../LoadMore.svelte";

  let { playlistId }: { playlistId?: string } = $props();
  let title = $state("");
  let image = $state("");
  let meta = $state("");
  let tracks = $state<Track[]>([]);
  let next = $state<string | null>(null);
  let busy = false;
  let gen = 0;
  let blocked = $state(false);

  $effect(() => {
    const pid = playlistId;
    const mine = ++gen;
    blocked = false;
    tracks = [];
    next = null;
    title = pid ? "" : "Liked Songs";
    image = "";
    meta = "";
    (async () => {
      try {
        if (pid) {
          const p = await get(`/playlists/${pid}?fields=name,images,owner(display_name)`);
          if (mine !== gen) return;
          title = p.name;
          image = img(p.images, 240);
          meta = p.owner?.display_name ?? "";
          // Spotify is moving playlist contents from /tracks to /items; try both.
          await loadPage(`/playlists/${pid}/items?limit=50&market=from_token`, mine).catch(() =>
            loadPage(`/playlists/${pid}/tracks?limit=50&market=from_token`, mine),
          );
        } else {
          await loadPage(`/me/tracks?limit=50&market=from_token`, mine);
        }
      } catch (e) {
        if (mine !== gen) return;
        if (errMsg(e).includes("403")) blocked = true;
        else toast.show(e);
      }
    })();
  });

  async function loadPage(url: string, mine: number) {
    busy = true;
    try {
      const page = await get(url);
      if (mine !== gen) return;
      const first = tracks.length === 0;
      tracks = [...tracks, ...unwrapTracks(page.items ?? [])];
      next = page.next;
      if (first) meta = [meta, `${page.total.toLocaleString()} songs`].filter(Boolean).join(" · ");
    } finally {
      busy = false;
    }
  }

  function more() {
    if (next && !busy) loadPage(next, gen).catch((e) => toast.show(e));
  }
</script>

<Header {image} kind={playlistId ? "Playlist" : "Collection"} title={title || "…"} {meta} onplay={tracks.length ? () => player.playList(tracks, 0) : undefined} />
{#if blocked}
  <p class="blocked">
    Spotify doesn't let personal apps like XSTIFY open this playlist. This usually happens with playlists made by
    Spotify or by other people. Playlists you created yourself work.
  </p>
{/if}
<TrackList {tracks} />
{#if next}<LoadMore onmore={more} />{/if}

<style>
  .blocked { color: var(--muted); max-width: 560px; line-height: 1.5; padding: 8px 0; }
</style>
