<script lang="ts">
  import "../app.css";
  import { applyTheme, loadTheme } from "$lib/theme";
  import TitleBar from "$lib/TitleBar.svelte";
  let { children } = $props();

  // Apply the saved colors before anything renders.
  applyTheme(loadTheme());
</script>

<div class="window">
  <TitleBar />
  <div class="content">{@render children()}</div>
</div>

<style>
  .window { height: 100vh; display: flex; flex-direction: column; }
  .content { flex: 1; min-height: 0; }
</style>
