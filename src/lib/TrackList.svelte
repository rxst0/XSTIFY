<script lang="ts">
  import { fmt, img, type Track } from "./api";
  import { nav, player } from "./state.svelte";
  import Icon from "./Icon.svelte";

  let {
    tracks,
    covers = true,
    album = true,
    numbered = false,
  }: { tracks: Track[]; covers?: boolean; album?: boolean; numbered?: boolean } = $props();
</script>

<div class="list" class:noalbum={!album}>
  {#each tracks as t, i (t.id + i)}
    {@const active = player.current?.uri === t.uri}
    <div
      class="row"
      class:active
      role="button"
      tabindex="0"
      title="Click to play"
      onclick={() => (active ? player.toggle() : player.playList(tracks, i))}
      onkeydown={(e) => e.key === "Enter" && player.playList(tracks, i)}
    >
      <span class="num">
        <span class="n">{#if active && player.playing}<Icon name="music" size={16} />{:else if numbered}{i + 1}{:else}<Icon name="play" size={16} />{/if}</span>
        <span class="p"><Icon name={active && player.playing ? "pause" : "play"} size={16} /></span>
      </span>
      <div class="title">
        {#if covers}<img src={img(t.album.images, 40)} alt="" loading="lazy" />{/if}
        <div class="text">
          <div class="name">{t.name}{#if t.explicit}<span class="e">E</span>{/if}</div>
          <div class="sub">
            {#each t.artists as a, j}<button class="link" onclick={(e) => { e.stopPropagation(); nav.go({ name: "artist", id: a.id }); }}>{a.name}</button>{#if j < t.artists.length - 1}, {/if}{/each}
          </div>
        </div>
      </div>
      {#if album}
        <button class="link albumcol" onclick={(e) => { e.stopPropagation(); nav.go({ name: "album", id: t.album.id }); }}>{t.album.name}</button>
      {/if}
      <div class="dur">{fmt(t.duration_ms)}</div>
    </div>
  {/each}
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 40px minmax(0, 4fr) minmax(0, 3fr) 56px;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
  }
  .noalbum .row { grid-template-columns: 40px minmax(0, 1fr) 56px; }
  .row:hover, .row:focus-visible { background: var(--hover); outline: none; }
  .row.active .name { color: var(--accent); }
  .num { color: var(--muted); display: grid; place-items: center; font-variant-numeric: tabular-nums; height: 32px; }
  .num .p { display: none; color: var(--accent); }
  .row:hover .num .n { display: none; }
  .row:hover .num .p { display: block; }
  .active .num .n { color: var(--accent); }
  .title { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .title img { width: 40px; height: 40px; border-radius: 4px; background: var(--surface-2); flex: none; }
  .text { min-width: 0; }
  .name, .sub, .albumcol { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .sub, .albumcol { color: var(--muted); font-size: 13px; }
  .albumcol { text-align: left; min-width: 0; }
  .e { font-size: 9px; background: var(--muted); color: var(--bg); border-radius: 2px; padding: 0 4px; margin-left: 6px; vertical-align: 2px; }
  .dur { color: var(--muted); font-size: 13px; text-align: right; font-variant-numeric: tabular-nums; }
  @media (max-width: 760px) {
    .row { grid-template-columns: 40px minmax(0, 1fr) 56px; }
    .albumcol { display: none; }
  }
</style>
