<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  let {
    needsClientId,
    busy,
    error,
    onlogin,
  }: { needsClientId: boolean; busy: boolean; error: string; onlogin: (clientId?: string) => void } = $props();
  let clientId = $state("");
</script>

<div class="wrap">
  <div class="box">
    <div class="brand">XSTIFY</div>
    {#if busy}
      <h1>Signing in…</h1>
      <p>Approve XSTIFY in the browser window that opened, then come back here.</p>
      <p>Seeing "Invalid redirect URI"? Your Spotify app's redirect URI must be exactly <code>http://127.0.0.1:8899/callback</code>.</p>
      <button class="ghost" onclick={() => invoke("cancel_login")}>Cancel</button>
    {:else}
      <h1>Connect your Spotify</h1>
      {#if needsClientId}
        <ol>
          <li>Go to <code>developer.spotify.com/dashboard</code> and click <b>Create app</b>.</li>
          <li>Add the redirect URI <code>http://127.0.0.1:8899/callback</code> and tick <b>Web API</b>.</li>
          <li>Copy the app's <b>Client ID</b> and paste it below.</li>
        </ol>
        <input bind:value={clientId} placeholder="Client ID" spellcheck="false" onkeydown={(e) => e.key === "Enter" && clientId.trim() && onlogin(clientId.trim())} />
        <button disabled={!clientId.trim()} onclick={() => onlogin(clientId.trim())}>Sign in with Spotify</button>
      {:else}
        <button onclick={() => onlogin()}>Sign in with Spotify</button>
      {/if}
    {/if}
    {#if error}<p class="err">{error}</p>{/if}
    <p class="note">Requires Spotify Premium.</p>
  </div>
</div>

<style>
  .wrap { height: 100vh; display: grid; place-items: center; padding: 16px; }
  .box { width: 100%; max-width: 460px; background: var(--surface); border-radius: 12px; padding: 32px; display: grid; gap: 14px; }
  .brand { font-weight: 800; letter-spacing: 0.12em; color: var(--accent); }
  h1 { font-size: 24px; margin: 0; }
  p, ol { color: var(--muted); margin: 0; line-height: 1.5; }
  ol { padding-left: 20px; display: grid; gap: 6px; }
  code { background: var(--surface-2); padding: 1px 6px; border-radius: 4px; color: var(--text); font-size: 13px; user-select: all; }
  input { background: var(--surface-2); border: 1px solid var(--surface-3); border-radius: 6px; padding: 12px; color: var(--text); font: inherit; outline: none; }
  input:focus { border-color: var(--text); }
  button { background: var(--accent); color: #000; font-weight: 700; padding: 12px; border-radius: 999px; }
  button:disabled { opacity: 0.5; cursor: default; }
  button.ghost { background: none; color: var(--text); border: 1px solid var(--surface-3); }
  button.ghost:hover { border-color: var(--text); }
  .err { color: #ff6b6b; }
  .note { font-size: 12px; }
</style>
