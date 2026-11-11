<script lang="ts">
  import type { Note } from '../../notes/types'
  import { ago } from '../../ui/ago'
  import { clock } from '../../ui/now.svelte'
  import Foot from '../Foot.svelte'
  import { bounds, read } from './scene'
  import Shapes from './Shapes.svelte'

  let { note }: { note: Note } = $props()
  const t = $derived(note.title.trim())
  const els = $derived(read(note.body))
  const view = $derived.by(() => {
    const b = bounds(els)
    if (!b) return ''
    const p = Math.max(b.w, b.h) * 0.06 + 8
    return `${b.x - p} ${b.y - p} ${b.w + p * 2} ${b.h + p * 2}`
  })
</script>

{#if t}<h3>{t}</h3>{/if}
<div class="pic">
  {#if view}
    <svg viewBox={view} preserveAspectRatio="xMidYMid meet" aria-hidden="true"><Shapes {els} thumb /></svg>
  {:else}
    <span>vazio</span>
  {/if}
</div>
<Foot icon="board">board · {ago(note.updated, clock.now)}</Foot>

<style>
  h3 {
    margin: 0 0 8px;
    font-size: 14px;
    line-height: 1.3;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pic {
    flex: 1;
    min-height: 0;
    display: grid;
    place-items: center;
  }

  svg {
    display: block;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  span {
    font: 11px var(--mono);
    color: var(--faint);
  }

  :global(.small) h3 {
    font-size: 13px;
  }
</style>
