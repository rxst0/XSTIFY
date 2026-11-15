<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { applyTheme, DEFAULT_THEME, loadTheme, PRESETS, saveTheme, type Theme } from "../theme";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ext, type ExtInfo } from "../spicetify.svelte";
  import { toast } from "../state.svelte";
  import { plan, type Plan } from "../plan.svelte";

  let theme = $state<Theme>(loadTheme());
  let version = $state("");
  getVersion().then((v) => (version = v)).catch(() => {});

  function set(t: Theme) {
    theme = { ...t };
    applyTheme(theme);
    saveTheme(theme);
  }

  // ---- Extensions
  let exts = $state<ExtInfo[]>([]);
  let changed = $state(false); // extensions only load/unload on restart
  invoke<ExtInfo[]>("ext_list").then((l) => (exts = l)).catch(() => {});

  async function addExt() {
    const picked = await open({ multiple: true, title: "Add Spicetify extensions", filters: [{ name: "Extension", extensions: ["js"] }] });
    if (!picked) return;
    try {
      exts = await invoke<ExtInfo[]>("ext_add", { paths: Array.isArray(picked) ? picked : [picked] });
      changed = true;
    } catch (e) {
      toast.show(e);
    }
  }
  async function toggleExt(x: ExtInfo) {
    exts = await invoke<ExtInfo[]>("ext_set_enabled", { name: x.name, enabled: !x.enabled });
    changed = true;
  }
  async function removeExt(x: ExtInfo) {
    exts = await invoke<ExtInfo[]>("ext_remove", { name: x.name });
    changed = true;
  }

  function switchPlan(p: Plan) {
    if (p === plan.value) return;
    if (p === "premium" && plan.product && plan.product !== "premium") {
      toast.show("Spotify says your account is Free. Premium mode needs a Premium account.");
      return;
    }
    plan.set(p);
    location.reload(); // the player starts differently in each mode
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
  <h2>Extensions</h2>
  <p class="hint">
    Add Spicetify extensions (<code>.js</code> files). XSTIFY runs them with its own compatibility layer: simple extensions
    work, but ones that rely on Spotify's private services or the official app's layout won't.
    <b>Only add extensions you trust</b>; they can do anything XSTIFY can.
  </p>

  {#if exts.length}
    <ul class="exts">
      {#each exts as x (x.name)}
        <li>
          <span class="xname">{x.name.replace(/\.js$/, "")}</span>
          <button class="switch" class:on={x.enabled} role="switch" aria-checked={x.enabled} title={x.enabled ? "On" : "Off"} onclick={() => toggleExt(x)}><span></span></button>
          <button class="rm" title="Remove" aria-label={`Remove ${x.name}`} onclick={() => removeExt(x)}>✕</button>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="row">
    <button class="ghost" onclick={addExt}>Add extension…</button>
    {#if changed}<button class="primary" onclick={() => location.reload()}>Restart XSTIFY to apply</button>{/if}
  </div>

  {#if ext.menu.length}
    <h3>Extension options</h3>
    <ul class="opts">
      {#each ext.menu as m}
        <li>
          <button class="opt" onclick={() => m.onClick(m)}>
            <span>{m.name}</span>
            {#if typeof m.isEnabled === "boolean"}<span class="switch" class:on={m.isEnabled}><span></span></span>{:else}<span class="chev">›</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<section>
  <h2>Account</h2>
  <p class="hint">
    {#if plan.isFree}
      <b>Free mode:</b> Spotify songs open in Spotify's official web player (with Spotify's ads). Local Files, themes, the
      equalizer for Local Files and extensions work inside XSTIFY.
    {:else}
      <b>Premium mode:</b> music plays right inside XSTIFY.
    {/if}
    {#if plan.product}
      Spotify says your account is <b>{plan.product === "premium" ? "Premium" : "Free"}</b>.
    {/if}
  </p>
  <div class="modes">
    <button class="mode" class:active={!plan.isFree} onclick={() => switchPlan("premium")}>Premium</button>
    <button class="mode" class:active={plan.isFree} onclick={() => switchPlan("free")}>Free</button>
  </div>
  {#if plan.isFree}
    <button class="ghost" onclick={() => plan.openWeb("")}>Open Spotify web player</button>
  {/if}
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
  .modes { display: inline-flex; background: var(--surface-2); border-radius: 999px; padding: 3px; margin: 0 8px 12px 0; }
  .mode { padding: 6px 16px; border-radius: 999px; font-weight: 700; color: var(--muted); }
  .mode.active { background: var(--accent); color: var(--on-accent); }
  section .ghost + .ghost { margin-left: 8px; }
  code { background: var(--surface-2); padding: 1px 5px; border-radius: 4px; font-size: 12px; }
  h3 { font-size: 14px; margin: 18px 0 8px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .row { display: flex; gap: 8px; flex-wrap: wrap; }
  .primary { background: var(--accent); color: var(--on-accent); padding: 8px 16px; border-radius: 999px; font-weight: 700; }
  .exts, .opts { list-style: none; margin: 0 0 12px; padding: 0; display: grid; gap: 4px; }
  .exts li { display: flex; align-items: center; gap: 10px; background: var(--surface-2); border-radius: 8px; padding: 8px 10px 8px 14px; }
  .xname { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; }
  .rm { width: 26px; height: 26px; border-radius: 50%; color: var(--muted); }
  .rm:hover { color: var(--accent); background: var(--surface-3); }
  .opt { width: 100%; display: flex; align-items: center; justify-content: space-between; gap: 10px; background: var(--surface-2); border-radius: 8px; padding: 10px 14px; text-align: left; }
  .opt:hover { background: var(--surface-3); }
  .chev { color: var(--accent); font-size: 18px; }
  .switch { width: 32px; height: 18px; border-radius: 999px; background: var(--surface-3); position: relative; flex: none; display: inline-block; }
  .switch > span { position: absolute; top: 2px; left: 2px; width: 14px; height: 14px; border-radius: 50%; background: var(--text); transition: transform 0.15s; }
  .switch.on { background: var(--accent); }
  .switch.on > span { transform: translateX(14px); background: var(--on-accent); }
</style>
