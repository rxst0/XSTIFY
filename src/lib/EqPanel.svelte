<script lang="ts">
  import { BANDS, eq, PRESETS } from "./eq.svelte";

  let { onclose }: { onclose: () => void } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />
<button class="backdrop" aria-label="Close equalizer" onclick={onclose}></button>

<div class="panel" role="dialog" aria-label="Equalizer">
  <div class="head">
    <b>Equalizer</b>
    <span class="preset">{eq.preset}</span>
    {#if !eq.enabled}<span class="off">Off · turn on with the switch</span>{/if}
  </div>

  <div class="bands" class:dim={!eq.enabled}>
    {#each BANDS as label, i}
      <label class="band">
        <span class="db">{eq.gains[i] > 0 ? "+" : ""}{eq.gains[i]}</span>
        <input
          type="range"
          min="-12"
          max="12"
          step="1"
          value={eq.gains[i]}
          oninput={(e) => eq.setBand(i, +e.currentTarget.value)}
          ondblclick={() => eq.setBand(i, 0)}
          aria-label={`${label} Hz`}
        />
        <span class="hz">{label}</span>
      </label>
    {/each}
  </div>

  <div class="presets">
    {#each Object.keys(PRESETS) as name}
      <button class:active={eq.preset === name} onclick={() => eq.applyPreset(name)}>{name}</button>
    {/each}
  </div>
  {#if !eq.available}
    <p class="warn">
      The equalizer needs a silent audio output to work (for example Voicemod's "Dummy Output"). None was found on this PC.
    </p>
  {:else}
    <p class="note">Double-click a slider to reset it. Only affects XSTIFY.</p>
  {/if}
</div>

<style>
  .backdrop { position: fixed; inset: 0; background: transparent; z-index: 20; cursor: default; }
  .panel {
    position: fixed;
    right: 16px;
    bottom: 92px;
    z-index: 21;
    width: min(480px, calc(100vw - 32px));
    background: var(--surface-2);
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.5);
  }
  .head { display: flex; align-items: baseline; gap: 10px; margin-bottom: 12px; }
  .preset { color: var(--accent); font-size: 13px; font-weight: 600; }
  .off { color: var(--muted); font-size: 12px; margin-left: auto; }
  .bands { display: grid; grid-template-columns: repeat(10, 1fr); gap: 4px; transition: opacity 0.15s; }
  .bands.dim { opacity: 0.5; }
  .band { display: grid; justify-items: center; gap: 6px; }
  .db { font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; min-height: 14px; }
  .hz { font-size: 11px; color: var(--muted); }
  input[type="range"] {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 20px;
    height: 140px;
    accent-color: var(--accent);
    cursor: pointer;
  }
  .presets { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 14px; }
  .presets button { font-size: 12px; font-weight: 600; padding: 5px 10px; border-radius: 999px; background: var(--surface-3); }
  .presets button:hover { filter: brightness(1.15); }
  .presets button.active { background: var(--accent); color: var(--on-accent); }
  .note { color: var(--muted); font-size: 11px; margin: 12px 0 0; }
  .warn { color: var(--text); font-size: 12px; margin: 12px 0 0; line-height: 1.4; }
</style>
