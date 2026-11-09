<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    image,
    title,
    subtitle = "",
    round = false,
    onclick,
  }: { image: string; title: string; subtitle?: string; round?: boolean; onclick: () => void } = $props();
</script>

<button class="card" {onclick}>
  <div class="art" class:round>
    {#if image}<img src={image} alt="" loading="lazy" />{:else}<Icon name="music" size={40} />{/if}
  </div>
  <div class="t">{title}</div>
  {#if subtitle}<div class="s">{subtitle}</div>{/if}
</button>

<style>
  .card { text-align: left; padding: 12px; border-radius: 8px; min-width: 0; }
  .card:hover, .card:focus-visible { background: var(--hover); }
  .art { aspect-ratio: 1; border-radius: 6px; overflow: hidden; background: var(--surface-2); display: grid; place-items: center; color: var(--muted); margin-bottom: 8px; }
  .art.round { border-radius: 50%; }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .t, .s { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .t { font-weight: 600; }
  .s { color: var(--muted); font-size: 13px; margin-top: 2px; }
</style>
