<script lang="ts">
  // Pop-up windows (Spicetify.PopupModal) and right-click menus (Spicetify.ContextMenu)
  // opened by extensions.
  import { ext } from "./spicetify.svelte";

  /** Puts an extension-provided element (or HTML string) into the modal. */
  function content(node: HTMLElement, c: HTMLElement | string) {
    const set = (v: HTMLElement | string) => {
      node.replaceChildren();
      if (typeof v === "string") node.innerHTML = v;
      else if (v) node.append(v);
    };
    set(c);
    return { update: set };
  }

  function run(i: (typeof ext.context)[number], uris: string[]) {
    ext.contextMenu = null;
    try {
      i.onClick(uris, [], "");
    } catch (e) {
      console.warn("[extension]", e);
    }
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") {
      ext.contextMenu = null;
      ext.modal = null;
    }
  }}
  onclick={() => (ext.contextMenu = null)}
  onblur={() => (ext.contextMenu = null)}
/>

{#if ext.contextMenu}
  {@const m = ext.contextMenu}
  <ul class="ctx" role="menu" style="left:{Math.min(m.x, innerWidth - 240)}px; top:{Math.min(m.y, innerHeight - 40 * m.items.length - 16)}px">
    {#each m.items as i}
      <li>
        <button role="menuitem" disabled={i.disabled} onclick={(e) => { e.stopPropagation(); run(i, m.uris); }}>{i.name}</button>
      </li>
    {/each}
  </ul>
{/if}

{#if ext.modal}
  <div class="backdrop" role="presentation" onclick={() => (ext.modal = null)}>
    <div class="modal" class:large={ext.modal.isLarge} role="dialog" aria-label={ext.modal.title} tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <header>
        <h2>{ext.modal.title}</h2>
        <button class="close" aria-label="Close" onclick={() => (ext.modal = null)}>✕</button>
      </header>
      <div class="body" use:content={ext.modal.content}></div>
    </div>
  </div>
{/if}

<style>
  .ctx { position: fixed; z-index: 40; min-width: 200px; max-width: 280px; margin: 0; padding: 4px; list-style: none; background: var(--surface-2); border-radius: 8px; box-shadow: 0 12px 32px rgb(0 0 0 / 0.5); }
  .ctx button { width: 100%; text-align: left; padding: 9px 12px; border-radius: 4px; font-size: 14px; }
  .ctx button:hover:not(:disabled) { background: var(--surface-3); }
  .ctx button:disabled { opacity: 0.5; }
  .backdrop { position: fixed; inset: 0; z-index: 30; background: rgb(0 0 0 / 0.6); display: grid; place-items: center; padding: 16px; }
  .modal { background: var(--surface); border-radius: 12px; width: min(520px, 100%); max-height: calc(100vh - 80px); display: flex; flex-direction: column; box-shadow: 0 20px 60px rgb(0 0 0 / 0.6); }
  .modal.large { width: min(860px, 100%); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 16px 20px 8px; }
  h2 { margin: 0; font-size: 20px; }
  .close { width: 32px; height: 32px; border-radius: 50%; color: var(--accent); }
  .close:hover { background: var(--hover); }
  .body { padding: 8px 20px 20px; overflow: auto; color: var(--text); }
  /* Make common extension markup readable in XSTIFY's theme */
  .body :global(button) { color: inherit; }
  .body :global(input), .body :global(select) { background: var(--surface-2); color: var(--text); border: 1px solid var(--surface-3); border-radius: 4px; padding: 4px 6px; }
</style>
