<script lang="ts">
  import "../app.css";
  import { applyTheme, loadTheme } from "$lib/theme";
  let { children } = $props();

  // Apply the saved colors before anything renders.
  applyTheme(loadTheme());
</script>

{@render children()}
