<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { applyTheme, DEFAULT_THEME, loadTheme, PRESETS, saveTheme, type Theme } from "../theme";

  let theme = $state<Theme>(loadTheme());
  let version = $state("");
  getVersion().then((v) => (version = v)).catch(() => {});

  function set(t: Theme) {
    theme = { ...t };
    applyTheme(theme);
    saveTheme(theme);
  }

  async function signOut() {
    await invoke("logout");
    location.reload();
  }
</script>

<h1>Settings</h1>

<section>
  <h2>Theme</h2>
  <p class="hint">Pick any two colors. <b>Main</b> sets the background and panels. <b>Accent</b> colors buttons, sliders and highlights.</p>

  <div class="pickers">
    <label>
      <input type="color" value={theme.main} oninput={(e) => set({ ...theme, main: e.currentTarget.value })} />
      <span><b>Main</b><small>{theme.main}</small></span>
    </label>
    <label>
      <input type="color" value={theme.accent} oninput={(e) => set({ ...theme, accent: e.currentTarget.value })} />
      <span><b>Accent</b><small>{theme.accent}</small></span>
    </label>
  </div>

  <div class="presets">
    {#each PRESETS as p}
      {@const active = p.main === theme.main && p.accent === theme.accent}
      <button class="preset" class:active onclick={() => set(p)} title={p.name}>
        <span class="swatch" style="background:{p.main}"><span class="dot" style="background:{p.accent}"></span></span>
        {p.name}
      </button>
    {/each}
  </div>

  <button class="ghost" onclick={() => set(DEFAULT_THEME)}>Reset to default</button>
</section>

<section>
  <h2>Account</h2>
  <button class="ghost" onclick={signOut}>Sign out</button>
</section>

<section>
  <h2>About</h2>
  <p class="hint">XSTIFY {version ? `v${version}` : ""} · unofficial, not affiliated with Spotify</p>
</section>

<style>
  h1 { font-size: 32px; margin: 8px 0 8px; }
  section { padding: 8px 0 16px; border-bottom: 1px solid var(--surface-2); max-width: 640px; }
  section:last-child { border-bottom: 0; }
  h2 { font-size: 18px; margin: 16px 0 6px; }
  .hint { color: var(--muted); margin: 0 0 14px; line-height: 1.5; }
  .pickers { display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 14px; }
  label { display: flex; align-items: center; gap: 10px; background: var(--surface-2); padding: 8px 14px 8px 8px; border-radius: 10px; cursor: pointer; }
  label span { display: grid; }
  label small { color: var(--muted); font-family: ui-monospace, monospace; text-transform: uppercase; }
  input[type="color"] { width: 40px; height: 40px; border: 0; padding: 0; background: none; cursor: pointer; }
  input[type="color"]::-webkit-color-swatch-wrapper { padding: 0; }
  input[type="color"]::-webkit-color-swatch { border: 2px solid var(--surface-3); border-radius: 8px; }
  .presets { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 14px; }
  .preset { display: flex; align-items: center; gap: 8px; padding: 6px 12px 6px 6px; border-radius: 999px; background: var(--surface-2); font-size: 13px; font-weight: 600; }
  .preset:hover { background: var(--surface-3); }
  .preset.active { outline: 2px solid var(--accent); }
  .swatch { width: 24px; height: 24px; border-radius: 50%; display: grid; place-items: center; border: 1px solid var(--surface-3); }
  .dot { width: 10px; height: 10px; border-radius: 50%; }
  .ghost { border: 1px solid var(--surface-3); padding: 8px 16px; border-radius: 999px; font-weight: 600; }
  .ghost:hover { border-color: var(--text); }
</style>
