<script lang="ts">
  // First-launch screen: which Spotify account do you have?
  import { plan, type Plan } from "./plan.svelte";

  let { onchosen }: { onchosen: () => void } = $props();

  function choose(p: Plan) {
    plan.set(p);
    onchosen();
  }
</script>

<div class="wrap">
  <div class="box">
    <h1>Which Spotify account do you have?</h1>
    <p class="sub">You can change this later in Settings.</p>

    <div class="cards">
      <button class="card" onclick={() => choose("premium")}>
        <span class="tag">Premium</span>
        <ul>
          <li>Music plays right inside XSTIFY</li>
          <li>Equalizer, media keys and Stream Deck for everything</li>
          <li>Local Files, themes, extensions</li>
        </ul>
      </button>
      <button class="card" onclick={() => choose("free")}>
        <span class="tag">Free</span>
        <ul>
          <li>Spotify songs open in Spotify's official web player (with Spotify's ads)</li>
          <li>Search, your library, Local Files, themes and extensions in XSTIFY</li>
          <li>Equalizer works on your Local Files</li>
        </ul>
      </button>
    </div>
    <p class="note">Not sure? Pick either. XSTIFY checks with Spotify after you sign in.</p>
  </div>
</div>

<style>
  .wrap { height: 100%; display: grid; place-items: center; padding: 16px; overflow: auto; }
  .box { width: 100%; max-width: 720px; display: grid; gap: 12px; }
  h1 { margin: 0; font-size: 28px; }
  .sub, .note { color: var(--muted); margin: 0; }
  .note { font-size: 13px; }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; margin: 8px 0; }
  .card { text-align: left; background: var(--surface); border: 2px solid var(--surface-2); border-radius: 14px; padding: 20px; display: grid; gap: 12px; align-content: start; transition: border-color 0.15s, transform 0.1s; }
  .card:hover { border-color: var(--accent); transform: translateY(-2px); }
  .tag { font-size: 20px; font-weight: 800; color: var(--accent); }
  ul { margin: 0; padding-left: 18px; color: var(--muted); display: grid; gap: 6px; line-height: 1.4; }
</style>
