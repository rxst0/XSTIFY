<script lang="ts">
  // Checks GitHub for a newer XSTIFY on startup. If found, shows one button that downloads,
  // verifies (signed), installs over the current version and restarts the app.
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { toast } from "./state.svelte";

  let update = $state<Update | null>(null);
  let progress = $state<number | null>(null);

  $effect(() => {
    check()
      .then((u) => (update = u))
      .catch(() => {}); // offline or no release yet: stay quiet
  });

  async function install() {
    if (!update || progress !== null) return;
    let total = 0;
    let done = 0;
    progress = 0;
    try {
      await update.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        else if (e.event === "Progress") {
          done += e.data.chunkLength;
          progress = total ? Math.round((done / total) * 100) : 0;
        }
      });
      // On Windows the installer closes XSTIFY, replaces it and starts the new version.
    } catch (e) {
      progress = null;
      toast.show(`Update failed: ${e}`);
    }
  }
</script>

{#if update}
  <button class="update" onclick={install} disabled={progress !== null} title={update.body ?? ""}>
    {#if progress === null}
      Update to v{update.version}
    {:else}
      Updating… {progress}%
    {/if}
  </button>
{/if}

<style>
  .update {
    margin: 0 8px 12px;
    padding: 8px 12px;
    border-radius: 999px;
    background: var(--accent);
    color: var(--on-accent);
    font-weight: 700;
    font-size: 13px;
    text-align: center;
  }
  .update:hover:not(:disabled) { filter: brightness(1.1); }
  .update:disabled { opacity: 0.8; cursor: default; }
</style>
