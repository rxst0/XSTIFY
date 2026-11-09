<script lang="ts">
  // Custom window title bar (the system one is turned off in tauri.conf.json).
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LOGO_INNER, LOGO_VIEWBOX, setTaskbarIcon } from "./logo";
  import { loadTheme } from "./theme";

  const win = getCurrentWindow();
  let maximized = $state(false);

  $effect(() => {
    const sync = () => win.isMaximized().then((m) => (maximized = m)).catch(() => {});
    sync();
    const unlisten = win.onResized(sync);
    // Keep the taskbar icon in the accent color, now and whenever the theme changes.
    setTaskbarIcon(loadTheme().accent);
    const onTheme = (e: Event) => setTaskbarIcon((e as CustomEvent<string>).detail);
    window.addEventListener("xstify-theme", onTheme);
    return () => {
      unlisten.then((f) => f());
      window.removeEventListener("xstify-theme", onTheme);
    };
  });
</script>

<header data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <svg viewBox={LOGO_VIEWBOX} width="18" height="18" aria-hidden="true" data-tauri-drag-region>{@html LOGO_INNER}</svg>
    <span data-tauri-drag-region>XSTIFY</span>
  </div>
  <div class="controls">
    <button title="Minimize" aria-label="Minimize" onclick={() => win.minimize()}>
      <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0 5h10" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
    <button title={maximized ? "Restore" : "Maximize"} aria-label={maximized ? "Restore" : "Maximize"} onclick={() => win.toggleMaximize()}>
      {#if maximized}
        <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.1"><path d="M2.5 2.5V.5h7v7h-2" /><rect x=".5" y="2.5" width="7" height="7" /></svg>
      {:else}
        <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.1"><rect x=".5" y=".5" width="9" height="9" /></svg>
      {/if}
    </button>
    <button class="close" title="Close" aria-label="Close" onclick={() => win.close()}>
      <svg viewBox="0 0 10 10" width="10" height="10"><path d="M0 0l10 10M10 0L0 10" stroke="currentColor" stroke-width="1.2" /></svg>
    </button>
  </div>
</header>

<style>
  header {
    height: 36px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg);
    user-select: none;
  }
  .brand { display: flex; align-items: center; gap: 8px; padding-left: 14px; color: var(--accent); height: 100%; }
  .brand span { font-weight: 800; font-size: 12px; letter-spacing: 0.14em; }
  .controls { display: flex; height: 100%; }
  .controls button { width: 46px; height: 100%; display: grid; place-items: center; color: var(--accent); }
  .controls button:hover { background: var(--hover); }
  .controls .close:hover { background: #e81123; color: #fff; }
</style>
