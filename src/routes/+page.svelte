<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { errMsg } from "$lib/api";
  import { nav, player, toast } from "$lib/state.svelte";
  import { eq } from "$lib/eq.svelte";
  import Login from "$lib/Login.svelte";
  import Sidebar from "$lib/Sidebar.svelte";
  import PlayerBar from "$lib/PlayerBar.svelte";
  import Icon from "$lib/Icon.svelte";
  import Search from "$lib/views/Search.svelte";
  import Artist from "$lib/views/Artist.svelte";
  import Album from "$lib/views/Album.svelte";
  import Collection from "$lib/views/Collection.svelte";
  import Settings from "$lib/views/Settings.svelte";

  type Phase = "starting" | "login" | "ready";
  let phase = $state<Phase>("starting");
  let needsClientId = $state(true);
  let busy = $state(false);
  let error = $state("");
  let searchEl = $state<HTMLInputElement>();
  let main: HTMLElement | undefined = $state();

  onMount(async () => {
    const s = await invoke<{ has_client_id: boolean; has_saved_login: boolean }>("status");
    needsClientId = !s.has_client_id;
    if (s.has_client_id && s.has_saved_login) await login();
    else phase = "login";
  });

  async function login(clientId?: string) {
    busy = true;
    error = "";
    try {
      await invoke("login", { clientId: clientId ?? null });
      await player.init();
      eq.restore();
      phase = "ready";
    } catch (e) {
      error = errMsg(e);
      phase = "login";
    } finally {
      busy = false;
    }
  }

  // Scroll to top when the view changes.
  $effect(() => {
    nav.view;
    main?.scrollTo(0, 0);
  });

  function onKey(e: KeyboardEvent) {
    if (phase !== "ready") return;
    const typing = e.target instanceof HTMLInputElement && e.target.type !== "range";
    if ((e.ctrlKey && e.key.toLowerCase() === "k") || (e.key === "/" && !typing)) {
      e.preventDefault();
      nav.go({ name: "search" });
      queueMicrotask(() => setTimeout(() => searchEl?.focus()));
    } else if (e.key === " " && !typing) {
      e.preventDefault();
      player.toggle();
    } else if (e.ctrlKey && e.key === "ArrowRight") player.next();
    else if (e.ctrlKey && e.key === "ArrowLeft") player.prev();
    else if (e.altKey && e.key === "ArrowLeft") nav.back();
    else if (e.key === "Escape" && typing) (e.target as HTMLInputElement).blur();
  }
</script>

<svelte:window onkeydown={onKey} onmouseup={(e) => e.button === 3 && nav.back()} />

{#if phase === "starting"}
  <div class="splash">XSTIFY</div>
{:else if phase === "login"}
  <Login {needsClientId} {busy} {error} onlogin={login} />
{:else}
  <div class="app">
    <Sidebar />
    <main bind:this={main}>
      {#if nav.stack.length > 1}
        <button class="back" title="Back (Alt+←)" onclick={() => nav.back()}><Icon name="back" /></button>
      {/if}
      {#if nav.view.name === "search"}
        <Search bind:searchEl />
      {:else if nav.view.name === "artist"}
        <Artist id={nav.view.id} />
      {:else if nav.view.name === "album"}
        <Album id={nav.view.id} />
      {:else if nav.view.name === "playlist"}
        <Collection playlistId={nav.view.id} />
      {:else if nav.view.name === "liked"}
        <Collection />
      {:else if nav.view.name === "settings"}
        <Settings />
      {/if}
    </main>
    <div class="bar"><PlayerBar /></div>
  </div>
{/if}

{#if toast.msg}<div class="toast" role="status">{toast.msg}</div>{/if}

<style>
  .splash { height: 100%; display: grid; place-items: center; font-weight: 800; letter-spacing: 0.2em; color: var(--accent); }
  .app { height: 100%; display: grid; grid-template-columns: 260px minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; gap: 8px; padding: 8px 8px 0; }
  main { background: var(--surface); border-radius: 10px; overflow-y: auto; padding: 16px 24px 32px; position: relative; }
  .bar { grid-column: 1 / -1; }
  .back { width: 32px; height: 32px; border-radius: 50%; background: var(--bg); color: var(--accent); display: grid; place-items: center; margin-bottom: 12px; }
  .back:hover { background: color-mix(in srgb, var(--accent) 18%, var(--bg)); }
  .toast { position: fixed; bottom: 96px; left: 50%; transform: translateX(-50%); background: var(--text); color: var(--bg); padding: 10px 16px; border-radius: 8px; font-size: 14px; max-width: min(560px, calc(100vw - 32px)); box-shadow: 0 8px 24px rgb(0 0 0 / 0.4); z-index: 10; }
  @media (max-width: 760px) { .app { grid-template-columns: 200px minmax(0, 1fr); } }
</style>
