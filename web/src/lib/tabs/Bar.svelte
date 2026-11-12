<script lang="ts">
  import Connect from '../devices/Connect.svelte'
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Kind } from '../notes/types'
  import Icon from '../ui/Icon.svelte'
  import { grow } from '../ui/motion.svelte'
  import { shell, type At } from '../shell'
  import Frame from '../shell/Frame.svelte'
  import Picker from './Picker.svelte'
  import { tabs } from './store.svelte'
  import { spot } from './tabs'

  let {
    picking = $bindable(false),
    onopen,
    onclose,
    ondetach,
    onhome,
    onpick,
    onscan,
    onfind,
  }: {
    picking?: boolean
    onopen: (id: string) => void
    onclose: (id: string) => void
    ondetach: (id: string, at: At) => void
    onhome: () => void
    onpick: (k: Kind) => void
    onscan: () => void
    onfind: () => void
  } = $props()
  let connecting = $state(false)

  // Quanto a aba precisa passar da barra, para baixo, para virar janela ao soltar.
  const AWAY = 48

  interface Drag {
    id: string
    i: number
    x: number
    y: number
    dx: number
    cx: number
    cy: number
    mids: number[]
    w: number
    to: number
    on: boolean
    out: boolean
  }

  let bar: HTMLElement
  let nav: HTMLElement
  let drag = $state<Drag | null>(null)
  let skip = false

  function down(e: PointerEvent, id: string, i: number) {
    skip = false
    if (e.button !== 0 || e.pointerType === 'touch' || (e.target as HTMLElement).closest('.x')) return
    const els = [...nav.querySelectorAll<HTMLElement>('.tab[data-id]')]
    const mids = els.map((el) => {
      const r = el.getBoundingClientRect()
      return r.left + r.width / 2
    })
    const w = els[i].getBoundingClientRect().width + 2
    drag = { id, i, x: e.clientX, y: e.clientY, dx: 0, cx: e.clientX, cy: e.clientY, mids, w, to: i, on: false, out: false }
    ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function move(e: PointerEvent) {
    const d = drag
    if (!d) return
    const dx = e.clientX - d.x
    if (!d.on && Math.hypot(dx, e.clientY - d.y) < 5) return
    const r = bar.getBoundingClientRect()
    const y = e.clientY
    d.on = true
    d.dx = dx
    d.cx = e.clientX
    d.cy = y
    d.out = shell.can.detach && (y > r.bottom + AWAY || y < 0 || e.clientX < 0 || e.clientX > innerWidth)
    d.to = d.out ? d.i : spot(d.mids, d.i, d.mids[d.i] + dx)
  }

  function up(e: PointerEvent) {
    const d = drag
    drag = null
    if (!d?.on) return
    skip = true
    if (d.out) ondetach(d.id, { x: e.screenX, y: e.screenY })
    else if (d.to !== d.i) tabs.move(d.id, d.to)
  }

  /** Quanto cada aba anda para abrir espaço para a arrastada. */
  function shift(j: number) {
    const d = drag
    if (!d?.on || d.out || j === d.i) return 0
    if (d.i < j && j <= d.to) return -d.w
    if (d.to <= j && j < d.i) return d.w
    return 0
  }
</script>

<header class="bar" class:away={!!tabs.on} bind:this={bar} data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region><span class="mark" data-tauri-drag-region></span>mindblow</div>
  <nav class="tabs" class:moving={drag?.on} bind:this={nav}>
    <button class="tab home" class:on={!tabs.on} onclick={onhome}>
      <Icon name="grid" size={14} /><span class="t">Notas</span><span class="n">{store.list.length}</span>
    </button>
    {#each tabs.ids as id, i (id)}
      {@const n = store.find(id)}
      {#if n}
        {@const s = kinds[n.kind]}
        {@const held = drag?.on && drag.id === id}
        <div
          class="tab"
          class:on={tabs.on === id}
          class:held
          class:gone={held && drag?.out}
          data-id={id}
          style:transform={held && !drag?.out ? `translateX(${drag?.dx}px)` : shift(i) ? `translateX(${shift(i)}px)` : null}
          role="tab"
          tabindex="0"
          aria-selected={tabs.on === id}
          transition:grow
          onpointerdown={(e) => down(e, id, i)}
          onpointermove={move}
          onpointerup={up}
          onpointercancel={() => (drag = null)}
          onclick={() => (skip ? (skip = false) : onopen(id))}
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
        onscan={() => {
          picking = false
          onscan()
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

{#if drag?.on && drag.out}
  {@const n = store.find(drag.id)}
  {#if n}
    <div class="ghost" style:left="{drag.cx}px" style:top="{drag.cy}px">
      <span class="gt"><Icon name={kinds[n.kind].icon} size={14} /><span>{kinds[n.kind].title(n)}</span></span>
      <span class="gh"><Icon name="detach" size={11} />solte para destacar</span>
    </div>
  {/if}
{/if}

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

  .moving .tab {
    transition:
      transform 160ms var(--ease),
      background 120ms,
      color 120ms;
  }

  .moving .tab.held {
    z-index: 1;
    background: var(--paper);
    color: var(--fg);
    box-shadow: var(--shadow-pop);
    cursor: grabbing;
    transition: none;
  }

  .tab.gone {
    opacity: 0.35;
    box-shadow: none;
  }

  .ghost {
    position: fixed;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 220px;
    padding: 12px 14px;
    border-radius: var(--r-lg);
    background: var(--paper);
    box-shadow: var(--shadow-pop);
    pointer-events: none;
    translate: -40px -16px;
  }

  .gt {
    display: flex;
    align-items: center;
    gap: 7px;
    overflow: hidden;
    font-size: 13px;
    white-space: nowrap;
  }

  .gt span {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .gh {
    display: flex;
    align-items: center;
    gap: 5px;
    font: 11px var(--mono);
    color: var(--faint);
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

    /* Só os ícones da toolbar: o popover do Connect mora dentro da barra, e o QR dele é um svg. */
    .add :global(svg),
    .find :global(svg),
    .qrb :global(svg) {
      width: 19px;
      height: 19px;
    }
  }
</style>
