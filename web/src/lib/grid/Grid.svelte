<script lang="ts">
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import { kinds, order as specs } from '../kinds'
  import { place } from '../notes/rank'
  import { store } from '../notes/store.svelte'
  import type { Kind, Note } from '../notes/types'
  import { shell } from '../shell'
  import { out } from '../shell/out.svelte'
  import Icon from '../ui/Icon.svelte'
  import { fly, pop } from '../ui/motion.svelte'
  import { near, shift, size, stack } from './layout'

  let { onopen }: { onopen: (id: string) => void } = $props()

  let only = $state<Kind | null>(null)
  let room = $state(0)
  let ready = $state(false)
  let order = $state<string[] | null>(null)
  let drag = $state<{ id: string; g: number; x: number; y: number; on: boolean } | null>(null)
  let el: HTMLDivElement

  const shown = $derived(only ? store.sorted.filter((n) => n.kind === only) : store.sorted)
  const groups = $derived([shown.filter((n) => n.pinned), shown.filter((n) => !n.pinned)])
  const view = $derived(groups.map((g, k) => (order && drag?.g === k ? arrange(g, order) : g)))
  const labels = $derived(groups[0].length > 0)
  const dims = $derived(size(room))
  const map = $derived(stack(view.map((g) => g.map((n) => Math.round(dims.w * kinds[n.kind].ratio))), dims, labels))
  const flat = $derived(view.flatMap((g, k) => g.map((n, i) => ({ n, k, b: map.boxes[k][i] }))))
  const count = $derived(Object.fromEntries(specs.map((s) => [s.kind, store.sorted.filter((n) => n.kind === s.kind).length])))
  const at = $derived(drag?.on ? view[drag.g].findIndex((n) => n.id === drag!.id) : -1)

  let start = { x: 0, y: 0, bx: 0, by: 0 }
  let hold: ReturnType<typeof setTimeout> | undefined

  onMount(() => {
    requestAnimationFrame(() => requestAnimationFrame(() => (ready = true)))
    const stop = (e: TouchEvent) => {
      if (drag?.on) e.preventDefault()
    }
    el.addEventListener('touchmove', stop, { passive: false })
    return () => el.removeEventListener('touchmove', stop)
  })

  function arrange(g: Note[], ids: string[]) {
    return ids.map((id) => g.find((n) => n.id === id)).filter((n): n is Note => !!n)
  }

  function down(e: PointerEvent, n: Note, k: number) {
    if (e.button !== 0 || (e.target as HTMLElement).closest('button')) return
    const b = map.boxes[k][view[k].indexOf(n)]
    start = { x: e.clientX, y: e.clientY, bx: b.x, by: b.y }
    drag = { id: n.id, g: k, x: b.x, y: b.y, on: false }
    if (e.pointerType === 'touch') hold = setTimeout(lift, 250)
    else (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  }

  function lift() {
    if (!drag) return
    drag.on = true
    order = view[drag.g].map((n) => n.id)
    navigator.vibrate?.(8)
  }

  function move(e: PointerEvent) {
    if (!drag) return
    const dx = e.clientX - start.x
    const dy = e.clientY - start.y
    if (!drag.on) {
      if (Math.hypot(dx, dy) < 6) return
      if (e.pointerType === 'touch') return cancel()
      lift()
    }
    drag.x = start.bx + dx
    drag.y = start.by + dy
    const boxes = map.boxes[drag.g]
    const h = boxes[at]?.h ?? 0
    const t = near(boxes, drag.x + dims.w / 2, drag.y + h / 2, dims.w)
    if (t >= 0 && t !== at && order) order = shift(order, at, t)
  }

  function up() {
    clearTimeout(hold)
    const d = drag
    if (!d) return
    drag = null
    if (!d.on) return onopen(d.id)
    const i = order?.indexOf(d.id) ?? -1
    order = null
    if (i >= 0) store.move(d.id, place(groups[d.g], d.id, i))
  }

  function cancel() {
    clearTimeout(hold)
    if (drag?.on) return up()
    drag = null
  }
</script>

<div class="wall">
  <div class="fit" bind:clientWidth={room}></div>
  <div class="inner" style:width="{map.width}px">
    <nav class="filter">
      <button class:on={!only} onclick={() => (only = null)}>Todas <span>{store.list.length}</span></button>
      {#each specs as s (s.kind)}
        <button class:on={only === s.kind} onclick={() => (only = only === s.kind ? null : s.kind)}>{s.label} <span>{count[s.kind]}</span></button>
      {/each}
      {#if dims.w >= 212 && store.list.length > 1}<span class="hint">arraste para reorganizar</span>{/if}
    </nav>

    {#if store.ready && !shown.length}
      <p class="empty" in:fade>{only ? 'nada deste tipo' : 'nada aqui ainda'} · <span class="kbd">n</span> cria uma nota</p>
    {/if}

    <div class="grid" class:ready class:small={dims.w < 212} style:height="{map.height}px" bind:this={el}>
      {#if labels}
        <div class="head" style:transform="translate(0px, {map.heads[0]}px)" transition:fade={{ duration: 160 }}>
          <Icon name="pin" size={11} />Fixadas
        </div>
        {#if groups[1].length}
          <div class="head" style:transform="translate(0px, {map.heads[1]}px)" transition:fade={{ duration: 160 }}>Outras</div>
        {/if}
      {/if}
      {#if drag && at >= 0}
        {@const b = map.boxes[drag.g][at]}
        <div class="slot" style:width="{dims.w}px" style:height="{b.h}px" style:transform="translate({b.x}px, {b.y}px)"></div>
      {/if}
      {#each flat as { n, k, b } (n.id)}
        {@const s = kinds[n.kind]}
        {@const lifted = drag?.on && drag.id === n.id}
        <div
          class="card {n.kind}"
          class:lift={lifted}
          style:width="{dims.w}px"
          style:height="{b.h}px"
          style:transform={lifted ? `translate(${drag!.x}px, ${drag!.y}px)` : `translate(${b.x}px, ${b.y}px)`}
          style:view-transition-name={fly.id === n.id ? `n-${n.id}` : null}
          in:pop
          out:fade={{ duration: 140 }}
          onpointerdown={(e) => down(e, n, k)}
          onpointermove={move}
          onpointerup={up}
          onpointercancel={cancel}
          onkeydown={(e) => e.key === 'Enter' && e.target === e.currentTarget && onopen(n.id)}
          tabindex="0"
          role="button"
          aria-label={s.title(n)}
        >
          <span class="acts">
            <button
              class="act"
              class:on={n.pinned}
              aria-label={n.pinned ? 'Desafixar' : 'Fixar'}
              title={n.pinned ? 'Desafixar' : 'Fixar'}
              onclick={() => store.pin(n.id, !n.pinned)}><Icon name="pin" size={14} /></button
            >
            {#if out.has(n.id)}
              <span class="badge"><Icon name="detach" size={11} />janela</span>
            {:else if shell.can.detach}
              <button class="act" aria-label="Destacar em janela" title="Destacar em janela" onclick={() => shell.detach(n)}
                ><Icon name="detach" size={14} /></button
              >
            {/if}
          </span>
          <s.Card note={n} />
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .wall {
    padding: 24px 40px 64px;
  }

  .inner {
    margin: 0 auto;
  }

  .filter {
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 20px;
    font-size: 13px;
  }

  .filter button {
    display: flex;
    align-items: baseline;
    gap: 6px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--muted);
    cursor: pointer;
  }

  .filter button:hover {
    color: var(--soft);
  }

  .filter button.on {
    color: var(--fg);
    font-weight: 500;
  }

  .filter span {
    font: 11px var(--mono);
    color: var(--faint);
  }

  .filter .hint {
    margin-left: auto;
  }

  .empty {
    margin: 18vh 0 0;
    text-align: center;
    font: 12px var(--mono);
    color: var(--faint);
  }

  .grid {
    position: relative;
  }

  .card {
    position: absolute;
    left: 0;
    top: 0;
    display: flex;
    flex-direction: column;
    border-radius: var(--r-lg);
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
    touch-action: manipulation;
    outline-offset: 3px;
    transition:
      box-shadow 200ms,
      border-color 120ms,
      translate 160ms var(--ease);
  }

  .ready .card {
    transition:
      transform 260ms var(--ease),
      box-shadow 200ms,
      border-color 120ms,
      translate 160ms var(--ease);
  }

  .card.text,
  .card.todo {
    padding: 16px 16px 14px;
    background: var(--paper);
    border: 1px solid var(--line);
    box-shadow: var(--shadow-card);
  }

  .card.sticky {
    padding: 18px;
    background: var(--sticky);
    border: 1px solid transparent;
    border-radius: var(--r-md);
  }

  .small .card.text,
  .small .card.todo {
    padding: 12px 12px 11px;
  }

  .small .card.sticky {
    padding: 14px;
  }

  @media (hover: hover) {
    .card:hover {
      translate: 0 -2px;
      border-color: var(--line-strong);
    }

    .card.sticky:hover {
      border-color: transparent;
    }

    .card:hover .act,
    .card:focus-within .act {
      opacity: 1;
    }
  }

  .card.lift {
    z-index: 10;
    cursor: grabbing;
    translate: 0 0;
    scale: 1.03;
    rotate: 1.5deg;
    box-shadow: var(--shadow-lift);
    transition:
      box-shadow 160ms,
      scale 160ms,
      rotate 160ms;
  }

  .acts {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .act {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: var(--bg);
    color: var(--muted);
    opacity: 0;
    cursor: pointer;
    transition:
      opacity 120ms,
      background 120ms,
      color 120ms;
  }

  .act:hover {
    color: var(--fg);
  }

  .act.on {
    background: var(--ink);
    color: var(--on-ink);
  }

  .act:focus-visible {
    opacity: 1;
  }

  .sticky .act:not(.on) {
    background: var(--paper);
  }

  .lift .act {
    opacity: 0 !important;
  }

  .badge {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 7px;
    border-radius: 999px;
    background: var(--ink);
    color: var(--on-ink);
    font: 10px var(--mono);
  }

  .head {
    position: absolute;
    left: 0;
    top: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    font: 500 11px var(--mono);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--faint);
  }

  .ready .head {
    transition: transform 260ms var(--ease);
  }

  .slot {
    position: absolute;
    left: 0;
    top: 0;
    border-radius: var(--r-lg);
    border: 1.5px dashed var(--line-strong);
    pointer-events: none;
    transition: transform 260ms var(--ease);
  }

  @media (max-width: 640px) {
    .wall {
      padding: 8px 18px 48px;
    }

    .filter {
      gap: 16px;
      overflow-x: auto;
    }

    .filter .hint {
      display: none;
    }
  }
</style>
