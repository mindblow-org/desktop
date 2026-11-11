<script lang="ts">
  import { onMount } from 'svelte'
  import { order } from '../kinds'
  import type { Kind } from '../notes/types'

  let { onpick, onclose }: { onpick: (k: Kind) => void; onclose: () => void } = $props()
  let el: HTMLDivElement
  let hot = $state<Kind>('text')

  onMount(() => {
    const key = (e: KeyboardEvent) => {
      const s = order.find((s) => s.key === e.key)
      if (s) {
        e.preventDefault()
        onpick(s.kind)
      } else if (e.key === 'Escape') {
        e.stopPropagation()
        onclose()
      }
    }
    const away = (e: PointerEvent) => {
      const t = e.target as HTMLElement
      if (!el.contains(t) && !t.closest('.add')) onclose()
    }
    addEventListener('keydown', key, true)
    addEventListener('pointerdown', away, true)
    return () => {
      removeEventListener('keydown', key, true)
      removeEventListener('pointerdown', away, true)
    }
  })
</script>

<div class="pop" bind:this={el} role="dialog" aria-label="Nova nota">
  <div class="head"><span>nova nota</span><span>esc fecha</span></div>
  <div class="kinds">
    {#each order as s (s.kind)}
      <button class="kind" class:on={hot === s.kind} onpointerenter={() => (hot = s.kind)} onclick={() => onpick(s.kind)}>
        <span class="shape">
          <span class="mini {s.kind}">
            {#if s.kind === 'sticky'}
              <i></i><i class="s"></i>
            {:else}
              <i class="h"></i>{#each [1, 2, 3, 4] as x (x)}<i class:d={s.kind === 'todo' && x === 1} class:s={x === 4}></i>{/each}
            {/if}
          </span>
        </span>
        <b>{s.label}</b>
        <span class="kbd">{s.key}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .pop {
    position: absolute;
    top: 38px;
    left: -10px;
    z-index: 50;
    width: 336px;
    padding: 14px;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-pop);
    transform-origin: 24px 0;
    animation: in 140ms var(--ease);
  }

  @keyframes in {
    from {
      opacity: 0;
      scale: 0.96;
    }
  }

  .head {
    display: flex;
    justify-content: space-between;
    margin: 0 2px 10px;
    font: 11px var(--mono);
    color: var(--faint);
  }

  .kinds {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }

  .kind {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 8px 12px;
    border: 0;
    border-radius: var(--r-md);
    background: none;
    color: var(--soft);
    cursor: pointer;
    transition: background 120ms;
  }

  .kind.on {
    background: var(--hover);
    color: var(--fg);
  }

  .kind b {
    font-size: 13px;
    font-weight: 500;
  }

  .shape {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    height: 66px;
  }

  .mini {
    display: flex;
    flex-direction: column;
    gap: 5px;
    width: 46px;
    padding: 9px 8px;
    border-radius: 5px;
    transition:
      translate 160ms var(--ease),
      box-shadow 160ms;
  }

  .kind.on .mini {
    translate: 0 -3px;
    box-shadow: var(--shadow-pop);
  }

  .mini.sticky {
    height: 46px;
    background: var(--sticky);
  }

  .mini.text,
  .mini.todo {
    height: 65px;
    background: var(--paper);
    box-shadow: inset 0 0 0 1px var(--line-strong);
  }

  .kind.on .mini.text,
  .kind.on .mini.todo {
    box-shadow:
      inset 0 0 0 1px var(--line-strong),
      var(--shadow-pop);
  }

  .mini i {
    display: block;
    height: 2px;
    border-radius: 1px;
    background: var(--line-strong);
  }

  .mini.sticky i {
    height: 3px;
    background: var(--faint);
  }

  .mini i.h {
    width: 70%;
    height: 3px;
    background: var(--muted);
  }

  .mini i.s {
    width: 55%;
  }

  .mini.todo i:not(.h) {
    display: flex;
    align-items: center;
    gap: 3px;
    height: 5px;
    background: none;
  }

  .mini.todo i:not(.h)::before {
    content: '';
    flex: none;
    width: 5px;
    height: 5px;
    border-radius: 1.5px;
    box-shadow: inset 0 0 0 1px var(--muted);
  }

  .mini.todo i:not(.h)::after {
    content: '';
    flex: 1;
    height: 2px;
    border-radius: 1px;
    background: var(--line-strong);
  }

  .mini.todo i.d::before {
    background: var(--ink);
    box-shadow: none;
  }

  @media (max-width: 640px) {
    .pop {
      position: fixed;
      top: auto;
      left: 12px;
      right: 12px;
      bottom: calc(12px + env(safe-area-inset-bottom));
      width: auto;
      transform-origin: 50% 100%;
    }
  }
</style>
