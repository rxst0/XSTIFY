<script lang="ts">
  import { get, img, type Playlist } from "./api";
  import { nav, toast } from "./state.svelte";
  import Icon from "./Icon.svelte";
  import UpdateButton from "./UpdateButton.svelte";

  let playlists = $state<Playlist[]>([]);

  $effect(() => {
    get("/me/playlists?limit=50")
      .then((r) => (playlists = (r.items ?? []).filter(Boolean)))
      .catch((e) => toast.show(e));
  });

  const isView = (name: string, id?: string) => nav.view.name === name && (id === undefined || (nav.view as any).id === id);
</script>

<aside>
  <div class="brand">XSTIFY</div>
  <UpdateButton />
  <nav>
    <button class:on={isView("search")} onclick={() => nav.go({ name: "search" })}><Icon name="search" /> Search</button>
    <button class:on={isView("liked")} onclick={() => nav.go({ name: "liked" })}><Icon name="heart" /> Liked Songs</button>
  </nav>
  <div class="label">Playlists</div>
  <div class="pls">
    {#each playlists as p (p.id)}
      <button class:on={isView("playlist", p.id)} onclick={() => nav.go({ name: "playlist", id: p.id })}>
        {#if p.images?.length}<img src={img(p.images, 32)} alt="" loading="lazy" />{:else}<span class="ph"><Icon name="music" size={16} /></span>{/if}
        <span class="pn">{p.name}</span>
      </button>
    {/each}
  </div>
  <button class="out" class:on={isView("settings")} onclick={() => nav.go({ name: "settings" })}><Icon name="settings" size={18} /> Settings</button>
</aside>

<style>
  aside { background: var(--surface); border-radius: 10px; display: flex; flex-direction: column; min-height: 0; padding: 16px 8px 8px; }
  .brand { font-weight: 800; letter-spacing: 0.12em; padding: 0 12px 14px; color: var(--accent); }
  nav { display: grid; gap: 2px; }
  button { display: flex; align-items: center; gap: 12px; width: 100%; padding: 8px 12px; border-radius: 6px; color: var(--muted); font-weight: 600; text-align: left; }
  button:hover { color: var(--text); background: var(--hover); }
  button.on { color: var(--text); background: var(--surface-2); }
  .label { color: var(--muted); font-size: 12px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; padding: 18px 12px 8px; }
  .pls { overflow-y: auto; flex: 1; min-height: 0; display: grid; align-content: start; gap: 2px; }
  .pls button { font-weight: 500; padding: 6px 12px; }
  .pls img, .ph { width: 32px; height: 32px; border-radius: 4px; flex: none; object-fit: cover; }
  .ph { background: var(--surface-2); display: grid; place-items: center; }
  .pn { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .out { font-size: 13px; margin-top: 6px; }
</style>
