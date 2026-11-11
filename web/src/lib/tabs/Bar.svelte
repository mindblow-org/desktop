<script lang="ts">
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Kind } from '../notes/types'
  import Icon from '../ui/Icon.svelte'
  import { grow } from '../ui/motion.svelte'
  import Picker from './Picker.svelte'
  import { tabs } from './store.svelte'
  import Sync from './Sync.svelte'

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

  const mac = /Mac|iPhone|iPad/.test(navigator.userAgent)
</script>

<header class="bar">
  <div class="brand"><span class="mark"></span>mindblow</div>
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
    <Icon name="search" size={14} /><span class="l">Buscar</span><span class="kbd">{mac ? '⌘ k' : 'ctrl k'}</span>
  </button>
  <Sync />
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
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    margin-left: auto;
    padding: 0 6px 0 10px;
    border: 0;
    border-radius: 999px;
    background: none;
    box-shadow: inset 0 0 0 1px var(--line);
    color: var(--muted);
    font-size: 12.5px;
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }

  .find:hover {
    background: var(--paper);
    color: var(--fg);
  }

  .bar > :global(.sync) {
    padding-left: 16px;
  }

  @media (max-width: 640px) {
    .bar {
      flex-wrap: wrap;
      height: auto;
      padding: env(safe-area-inset-top) 14px 0 18px;
      row-gap: 0;
    }

    .brand {
      height: 48px;
      order: 0;
    }

    .find {
      order: 1;
      width: 34px;
      height: 34px;
      padding: 0;
      justify-content: center;
    }

    .find .l,
    .find .kbd {
      display: none;
    }

    .bar > :global(.sync) {
      order: 1;
      padding: 0 12px;
    }

    .addw {
      order: 2;
    }

    .add {
      width: 34px;
      height: 34px;
      background: var(--paper);
      box-shadow: 0 0 0 1px var(--line);
      color: var(--fg);
    }

    .tabs {
      order: 3;
      flex-basis: 100%;
      margin: 0 -14px 0 -4px;
      padding: 2px 14px 10px 0;
    }

    .tab {
      height: 32px;
    }
  }
</style>
