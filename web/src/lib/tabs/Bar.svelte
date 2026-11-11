<script lang="ts">
  import Connect from '../devices/Connect.svelte'
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Kind } from '../notes/types'
  import Icon from '../ui/Icon.svelte'
  import { grow } from '../ui/motion.svelte'
  import Frame from '../shell/Frame.svelte'
  import Picker from './Picker.svelte'
  import { tabs } from './store.svelte'

  let {
    picking = $bindable(false),
    onopen,
    onclose,
    onhome,
    onpick,
    onfind,
  }: {
    picking?: boolean
    onopen: (id: string) => void
    onclose: (id: string) => void
    onhome: () => void
    onpick: (k: Kind) => void
    onfind: () => void
  } = $props()
  let connecting = $state(false)
</script>

<header class="bar" class:away={!!tabs.on} data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region><span class="mark" data-tauri-drag-region></span>mindblow</div>
  <nav class="tabs">
    <button class="tab home" class:on={!tabs.on} onclick={onhome}>
      <Icon name="grid" size={14} /><span class="t">Notas</span><span class="n">{store.list.length}</span>
    </button>
    {#each tabs.ids as id (id)}
      {@const n = store.find(id)}
      {#if n}
        {@const s = kinds[n.kind]}
        <div
          class="tab"
          class:on={tabs.on === id}
          role="tab"
          tabindex="0"
          aria-selected={tabs.on === id}
          transition:grow
          onclick={() => onopen(id)}
          onauxclick={(e) => e.button === 1 && onclose(id)}
          onkeydown={(e) => e.key === 'Enter' && onopen(id)}
        >
          <Icon name={s.icon} size={14} /><span class="t">{s.title(n)}</span>
          <button
            class="x"
            aria-label="Fechar aba"
            onclick={(e) => {
              e.stopPropagation()
              onclose(id)
            }}><Icon name="x" size={12} /></button
          >
        </div>
      {/if}
    {/each}
  </nav>
  <div class="addw">
    <button class="add" class:on={picking} aria-label="Nova nota" title="Nova nota  N" onclick={() => (picking = !picking)}>
      <Icon name="plus" />
    </button>
    {#if picking}
      <Picker
        onpick={(k) => {
          picking = false
          onpick(k)
        }}
        onclose={() => (picking = false)}
      />
    {/if}
  </div>
  <button class="find" aria-label="Buscar notas" title="Buscar notas" onclick={onfind}>
    <Icon name="search" size={18} />
  </button>
  <div class="qrw">
    <button class="qrb" class:on={connecting} aria-label="Conectar outro aparelho" title="Conectar outro aparelho" onclick={() => (connecting = !connecting)}>
      <Icon name="qr" size={15} />
    </button>
    {#if connecting}<Connect onclose={() => (connecting = false)} />{/if}
  </div>
  <Frame />
</header>

<style>
  .bar {
    position: relative;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 2px;
    height: 44px;
    padding: 0 12px 0 16px;
    padding-top: env(safe-area-inset-top);
    box-sizing: content-box;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-right: 16px;
    font: 500 13px var(--mono);
    letter-spacing: -0.02em;
  }

  .mark {
    width: 12px;
    height: 12px;
    border-radius: 4px;
    background: var(--ink);
    box-shadow: 3px 3px 0 -1px var(--line-strong);
  }

  .tabs {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .tab {
    flex: none;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    max-width: 210px;
    padding: 0 10px;
    overflow: hidden;
    border: 0;
    border-radius: var(--r-md);
    background: none;
    color: var(--muted);
    font-size: 13px;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }

  .tab .t {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab .n {
    font: 11px var(--mono);
    color: var(--faint);
  }

  .tab:hover {
    background: var(--hover);
    color: var(--soft);
  }

  .tab.on {
    background: var(--paper);
    color: var(--fg);
    box-shadow:
      var(--shadow-card),
      0 0 0 1px var(--line);
  }

  .x {
    display: grid;
    place-items: center;
    flex: none;
    width: 0;
    height: 18px;
    margin: 0;
    padding: 0;
    overflow: hidden;
    border: 0;
    border-radius: 5px;
    background: none;
    color: var(--faint);
    opacity: 0;
    cursor: pointer;
    transition:
      opacity 120ms,
      width 160ms var(--ease),
      margin 160ms var(--ease);
  }

  .tab:hover .x,
  .tab.on .x,
  .x:focus-visible {
    width: 18px;
    margin: 0 -4px 0 2px;
    opacity: 1;
  }

  .x:hover {
    background: var(--hover);
    color: var(--fg);
  }

  .addw {
    position: relative;
    flex: none;
    margin-left: 4px;
  }

  .add {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: none;
    color: var(--muted);
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }

  .add:hover,
  .add.on {
    background: var(--paper);
    color: var(--fg);
    box-shadow: 0 0 0 1px var(--line);
  }

  .find {
    display: none;
  }

  .qrw {
    position: relative;
    flex: none;
    margin-left: auto;
  }

  .qrb {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: none;
    color: var(--muted);
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }

  .qrb:hover,
  .qrb.on {
    background: var(--paper);
    color: var(--fg);
    box-shadow: 0 0 0 1px var(--line);
  }

  /* No celular a barra vira uma toolbar flutuante embaixo, no centro: QR, nova nota e busca. Com uma nota aberta, quem aparece é a da nota. Centraliza sem transform, que prenderia o Picker e o Connect (fixed) dentro dela. */
  @media (max-width: 640px) {
    .bar {
      position: fixed;
      left: 0;
      right: 0;
      bottom: calc(16px + env(safe-area-inset-bottom));
      width: fit-content;
      height: 48px;
      margin: 0 auto;
      padding: 0 4px;
      gap: 2px;
      border-radius: 999px;
      background: var(--ink);
      box-shadow: var(--shadow-pop);
    }

    .bar.away,
    .brand,
    .tabs,
    .bar > :global(.frame) {
      display: none;
    }

    .qrw {
      order: 0;
      margin: 0;
    }

    .addw {
      order: 1;
      margin: 0;
    }

    .find {
      order: 2;
      display: grid;
      place-items: center;
      padding: 0;
      border: 0;
      cursor: pointer;
    }

    .qrb,
    .add,
    .find {
      width: 48px;
      height: 40px;
      border-radius: 999px;
      background: none;
      box-shadow: none;
      color: var(--on-ink);
    }

    .qrb:hover,
    .qrb.on,
    .add:hover,
    .add.on,
    .find:hover {
      background: color-mix(in srgb, var(--on-ink) 16%, transparent);
      box-shadow: none;
      color: var(--on-ink);
    }

    .bar :global(svg) {
      width: 19px;
      height: 19px;
    }
  }
</style>
