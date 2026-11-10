<script lang="ts">
  import { fmt, img } from "./api";
  import { nav, player } from "./state.svelte";
  import { eq } from "./eq.svelte";
  import Icon from "./Icon.svelte";
  import EqPanel from "./EqPanel.svelte";

  let dragging = $state<number | null>(null);
  let lastVol = 70;
  let eqOpen = $state(false);

  let t = $derived(player.current);
  let dur = $derived(t?.duration_ms ?? 0);
  let pos = $derived(dragging ?? Math.min(player.position, dur));

  eq.refresh();

  function openEq() {
    eqOpen = !eqOpen;
    if (eqOpen) eq.refresh();
  }

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
      {#if t.album.id}
        <button class="cover" title="Go to album" onclick={() => nav.go({ name: "album", id: t.album.id })}>
          <img src={img(t.album.images, 64)} alt="" />
        </button>
      {:else}
        <!-- Local file: no Spotify album page -->
        <button class="cover local" title="Local Files" onclick={() => nav.go({ name: "local" })}><Icon name="folder" size={26} /></button>
      {/if}
      <div class="meta">
        <div class="name">{t.name}</div>
        <div class="artists">
          {#each t.artists as a, j}{#if a.id}<button class="link" onclick={() => nav.go({ name: "artist", id: a.id })}>{a.name}</button>{:else}{a.name}{/if}{#if j < t.artists.length - 1}, {/if}{/each}
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
    <div class="eq">
      <button class="eqbtn" class:open={eqOpen} title="Equalizer" onclick={openEq}>
        <Icon name="bass" size={18} /><span>EQ</span>
      </button>
      <button
        class="switch"
        class:on={eq.enabled}
        disabled={eq.busy}
        role="switch"
        aria-checked={eq.enabled}
        title={eq.enabled ? "Equalizer on" : "Equalizer off"}
        onclick={() => eq.setEnabled(!eq.enabled)}
      ><span></span></button>
    </div>
    <button class="icon" title="Mute" onclick={toggleMute}><Icon name={player.volume > 0 ? "volume" : "mute"} /></button>
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

{#if eqOpen}<EqPanel onclose={() => (eqOpen = false)} />{/if}

<style>
  footer { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.6fr) minmax(0, 1fr); align-items: center; gap: 16px; padding: 8px 12px; height: 76px; }
  .now { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .cover img { width: 56px; height: 56px; border-radius: 4px; display: block; background: var(--surface-2); }
  .cover.local { width: 56px; height: 56px; border-radius: 4px; background: var(--surface-2); color: var(--accent); display: grid; place-items: center; flex: none; }
  .meta { min-width: 0; }
  .name, .artists { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name { font-weight: 600; font-size: 14px; }
  .artists { color: var(--muted); font-size: 12px; }

  .center { display: grid; gap: 6px; justify-items: center; }
  .buttons { display: flex; align-items: center; gap: 18px; }
  .buttons button { color: var(--accent); display: grid; place-items: center; transition: transform 0.1s, filter 0.1s; }
  .buttons button:hover:not(:disabled) { filter: brightness(1.2); transform: scale(1.08); }
  .buttons button:disabled { opacity: 0.35; cursor: default; }
  .buttons .pp { width: 38px; height: 38px; border-radius: 50%; background: var(--accent); color: var(--on-accent); }
  .seek { display: flex; align-items: center; gap: 8px; width: 100%; max-width: 600px; font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; }
  .seek span { width: 36px; text-align: center; }
  .seek input { flex: 1; }

  .right { display: flex; justify-content: flex-end; align-items: center; gap: 8px; }
  .right > input { width: 110px; }
  .icon { color: var(--accent); display: grid; place-items: center; }
  .icon:hover { filter: brightness(1.2); }

  .eq { display: flex; align-items: center; gap: 6px; margin-right: 6px; }
  .eqbtn { display: flex; align-items: center; gap: 4px; color: var(--accent); font-size: 12px; font-weight: 700; padding: 4px 8px; border-radius: 999px; }
  .eqbtn:hover, .eqbtn.open { background: color-mix(in srgb, var(--accent) 18%, transparent); }
  .switch { width: 32px; height: 18px; border-radius: 999px; background: var(--surface-3); position: relative; transition: background 0.15s; flex: none; }
  .switch span { position: absolute; top: 2px; left: 2px; width: 14px; height: 14px; border-radius: 50%; background: var(--text); transition: transform 0.15s, background 0.15s; }
  .switch.on { background: var(--accent); }
  .switch.on span { transform: translateX(14px); background: var(--on-accent); }

  .spin { width: 16px; height: 16px; border: 2px solid var(--on-accent); border-top-color: transparent; border-radius: 50%; animation: s 0.7s linear infinite; }
  @keyframes s { to { transform: rotate(360deg); } }

  input[type="range"] { appearance: none; height: 4px; border-radius: 2px; background: linear-gradient(to right, var(--accent) var(--p), var(--surface-3) var(--p)); cursor: pointer; }
  input[type="range"]::-webkit-slider-thumb { appearance: none; width: 12px; height: 12px; border-radius: 50%; background: var(--accent); opacity: 0; }
  input[type="range"]:hover::-webkit-slider-thumb { opacity: 1; }
  input[type="range"]:disabled { cursor: default; }
  @media (max-width: 760px) { .right { display: none; } footer { grid-template-columns: minmax(0, 1fr) minmax(0, 1.4fr); } }
</style>
