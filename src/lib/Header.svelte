<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let {
    image,
    kind,
    title,
    meta = "",
    round = false,
    onplay,
    children,
  }: {
    image: string;
    kind: string;
    title: string;
    meta?: string;
    round?: boolean;
    onplay?: () => void;
    children?: Snippet;
  } = $props();
</script>

<header>
  <div class="art" class:round>
    {#if image}<img src={image} alt="" />{:else}<Icon name="music" size={64} />{/if}
  </div>
  <div class="info">
    <div class="kind">{kind}</div>
    <h1>{title}</h1>
    <div class="meta">{@render children?.()}{meta}</div>
  </div>
</header>
{#if onplay}
  <button class="play" title="Play" onclick={onplay}><Icon name="play" size={28} /></button>
{/if}

<style>
  header { display: flex; align-items: flex-end; gap: 24px; padding: 8px 0 20px; }
  .art { width: 200px; height: 200px; flex: none; border-radius: 8px; overflow: hidden; background: var(--surface-2); display: grid; place-items: center; color: var(--muted); box-shadow: 0 8px 32px rgb(0 0 0 / 0.4); }
  .art.round { border-radius: 50%; }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .info { min-width: 0; }
  .kind { font-size: 13px; font-weight: 600; }
  h1 { font-size: clamp(28px, 5vw, 64px); line-height: 1.05; margin: 6px 0 10px; overflow-wrap: anywhere; }
  .meta { color: var(--muted); font-size: 14px; }
  .play { width: 56px; height: 56px; border-radius: 50%; background: var(--accent); color: #000; display: grid; place-items: center; margin: 4px 0 8px; transition: transform 0.1s; }
  .play:hover { transform: scale(1.05); }
  @media (max-width: 640px) {
    header { flex-direction: column; align-items: flex-start; }
    .art { width: 140px; height: 140px; }
  }
</style>
