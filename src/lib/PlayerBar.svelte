<script lang="ts">
  import { fmt, img } from "./api";
  import { nav, player } from "./state.svelte";
  import Icon from "./Icon.svelte";

  let dragging = $state<number | null>(null);
  let lastVol = 70;

  let t = $derived(player.current);
  let dur = $derived(t?.duration_ms ?? 0);
  let pos = $derived(dragging ?? Math.min(player.position, dur));

  function toggleMute() {
    if (player.volume > 0) {
      lastVol = player.volume;
      player.setVolume(0);
    } else player.setVolume(lastVol || 70);
  }
</script>

<footer>
  <div class="now">
    {#if t}
      <button class="cover" title="Go to album" onclick={() => nav.go({ name: "album", id: t.album.id })}>
        <img src={img(t.album.images, 64)} alt="" />
      </button>
      <div class="meta">
        <div class="name">{t.name}</div>
        <div class="artists">
          {#each t.artists as a, j}<button class="link" onclick={() => nav.go({ name: "artist", id: a.id })}>{a.name}</button>{#if j < t.artists.length - 1}, {/if}{/each}
        </div>
      </div>
    {/if}
  </div>

  <div class="center">
    <div class="buttons">
      <button title="Previous (Ctrl+←)" onclick={() => player.prev()} disabled={!t}><Icon name="prev" /></button>
      <button class="pp" title="Play/Pause (Space)" onclick={() => player.toggle()} disabled={!t}>
        {#if player.loading}<span class="spin"></span>{:else}<Icon name={player.playing ? "pause" : "play"} size={22} />{/if}
      </button>
      <button title="Next (Ctrl+→)" onclick={() => player.next()} disabled={!t || !player.hasNext}><Icon name="next" /></button>
    </div>
    <div class="seek">
      <span>{fmt(pos)}</span>
      <input
        type="range"
        min="0"
        max={dur || 1}
        value={pos}
        disabled={!t}
        style="--p:{dur ? (pos / dur) * 100 : 0}%"
        oninput={(e) => (dragging = +e.currentTarget.value)}
        onchange={(e) => {
          player.seek(+e.currentTarget.value);
          dragging = null;
        }}
      />
      <span>{fmt(dur)}</span>
    </div>
  </div>

  <div class="right">
    <button title="Mute" onclick={toggleMute}><Icon name={player.volume > 0 ? "volume" : "mute"} /></button>
    <input
      type="range"
      min="0"
      max="100"
      value={player.volume}
      style="--p:{player.volume}%"
      oninput={(e) => player.setVolume(+e.currentTarget.value)}
      aria-label="Volume"
    />
  </div>
</footer>

<style>
  footer { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.6fr) minmax(0, 1fr); align-items: center; gap: 16px; padding: 8px 12px; height: 76px; }
  .now { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .cover img { width: 56px; height: 56px; border-radius: 4px; display: block; background: var(--surface-2); }
  .meta { min-width: 0; }
  .name, .artists { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name { font-weight: 600; font-size: 14px; }
  .artists { color: var(--muted); font-size: 12px; }
  .center { display: grid; gap: 6px; justify-items: center; }
  .buttons { display: flex; align-items: center; gap: 18px; }
  .buttons button { color: var(--muted); display: grid; place-items: center; }
  .buttons button:hover:not(:disabled) { color: var(--text); }
  .buttons button:disabled { opacity: 0.4; cursor: default; }
  .buttons .pp { width: 36px; height: 36px; border-radius: 50%; background: var(--text); color: var(--bg); }
  .buttons .pp:hover:not(:disabled) { color: var(--bg); transform: scale(1.05); }
  .seek { display: flex; align-items: center; gap: 8px; width: 100%; max-width: 600px; font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .seek span { width: 36px; text-align: center; }
  .seek input { flex: 1; }
  .right { display: flex; justify-content: flex-end; align-items: center; gap: 8px; color: var(--muted); }
  .right input { width: 110px; }  .right button:hover { color: var(--text); }
  .spin { width: 16px; height: 16px; border: 2px solid var(--bg); border-top-color: transparent; border-radius: 50%; animation: s 0.7s linear infinite; }
  @keyframes s { to { transform: rotate(360deg); } }

  input[type="range"] { appearance: none; height: 4px; border-radius: 2px; background: linear-gradient(to right, var(--text) var(--p), var(--surface-3) var(--p)); cursor: pointer; }
  input[type="range"]:hover { background: linear-gradient(to right, var(--accent) var(--p), var(--surface-3) var(--p)); }
  input[type="range"]::-webkit-slider-thumb { appearance: none; width: 12px; height: 12px; border-radius: 50%; background: var(--text); opacity: 0; }
  input[type="range"]:hover::-webkit-slider-thumb { opacity: 1; }
  input[type="range"]:disabled { cursor: default; }
  @media (max-width: 760px) { .right { display: none; } footer { grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr); } }
</style>
