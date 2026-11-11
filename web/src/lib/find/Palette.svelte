<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { fade } from 'svelte/transition'
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Note } from '../notes/types'
  import { ago } from '../ui/ago'
  import Icon from '../ui/Icon.svelte'
  import { clock } from '../ui/now.svelte'
  import { search, type Part } from './match'

  let { onopen, onclose }: { onopen: (id: string) => void; onclose: () => void } = $props()
  let q = $state('')
  let at = $state(0)
  let input: HTMLInputElement
  let list = $state<HTMLUListElement>()
  const hits = $derived(search(store.sorted, q, (n) => kinds[n.kind].title(n)))

  onMount(() => input.focus())

  function meta(n: Note) {
    const k = kinds[n.kind].label.toLowerCase()
    if (n.kind === 'todo') return `${k} · ${n.items.filter((i) => i.done).length}/${n.items.length}`
    return `${k} · ${ago(n.updated, clock.now)}`
  }

  async function go(i: number) {
    at = Math.max(0, Math.min(i, hits.length - 1))
    await tick()
    list?.querySelector('.on')?.scrollIntoView({ block: 'nearest' })
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      void go(at + 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      void go(at - 1)
    } else if (e.key === 'Enter' && hits[at]) {
      e.preventDefault()
      onopen(hits[at].note.id)
    } else if (e.key === 'Escape') {
      e.preventDefault()
      e.stopPropagation()
      onclose()
    }
  }
</script>

{#snippet parts(ps: Part[])}
  {#each ps as p, i (i)}{#if p.on}<mark>{p.t}</mark>{:else}{p.t}{/if}{/each}
{/snippet}

<div class="scrim" role="presentation" onclick={onclose} transition:fade={{ duration: 140 }}></div>
<div class="pal" role="dialog" aria-label="Buscar notas">
  <label class="q">
    <Icon name="search" size={18} />
    <input
      bind:this={input}
      bind:value={q}
      oninput={() => (at = 0)}
      onkeydown={key}
      placeholder="Buscar notas"
      spellcheck="false"
      autocomplete="off"
      aria-controls="hits"
    />
    <span class="kbd">esc</span>
  </label>
  {#if hits.length}
    <ul class="res" id="hits" role="listbox" bind:this={list}>
      {#each hits as h, i (h.note.id)}
        <li role="option" aria-selected={i === at}>
          <button class="r" class:on={i === at} onpointermove={() => (at = i)} onclick={() => onopen(h.note.id)}>
            <span class="k {h.note.kind}"><Icon name={kinds[h.note.kind].icon} size={14} /></span>
            <span class="b">
              <span class="t">
                {@render parts(h.title)}
                {#if h.note.pinned}<span class="pinned"><Icon name="pin" size={11} /></span>{/if}
              </span>
              {#if h.snip.length}<span class="s">{@render parts(h.snip)}</span>{/if}
            </span>
            <span class="m">{meta(h.note)}</span>
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="none">{store.list.length ? 'nada encontrado' : 'nenhuma nota ainda'}</p>
  {/if}
  <footer>
    <span><span class="kbd">↑</span><span class="kbd">↓</span>navegar</span>
    <span><span class="kbd">enter</span>abrir</span>
    <span class="wide">sem diferenciar acentos</span>
    <span class="n">{hits.length} {hits.length === 1 ? 'nota' : 'notas'}</span>
  </footer>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: color-mix(in srgb, var(--bg) 55%, transparent);
    backdrop-filter: blur(2px);
  }

  .pal {
    position: fixed;
    left: 50%;
    top: 14vh;
    z-index: 61;
    width: min(600px, calc(100vw - 24px));
    translate: -50% 0;
    overflow: hidden;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-pop);
    transform-origin: 50% 0;
    animation: in 160ms var(--ease);
  }

  @keyframes in {
    from {
      opacity: 0;
      scale: 0.97;
      translate: -50% -6px;
    }
  }

  .q {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 54px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
    color: var(--muted);
  }

  .q input {
    flex: 1;
    min-width: 0;
    height: 100%;
    font-size: 16px;
    color: var(--fg);
  }

  .res {
    max-height: 52vh;
    margin: 0;
    padding: 6px;
    overflow: auto;
    list-style: none;
  }

  .r {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px;
    border: 0;
    border-radius: var(--r-md);
    background: none;
    text-align: left;
    cursor: pointer;
  }

  .r.on {
    background: var(--hover);
  }

  .k {
    display: grid;
    place-items: center;
    flex: none;
    width: 30px;
    height: 30px;
    border-radius: var(--r-sm);
    background: var(--bg);
    color: var(--muted);
  }

  .r.on .k {
    background: var(--paper);
  }

  .k.sticky,
  .r.on .k.sticky {
    background: var(--sticky);
  }

  .b {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .t,
  .s {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .t {
    font-size: 14px;
    font-weight: 500;
  }

  .s {
    font-size: 12.5px;
    color: var(--muted);
  }

  .pinned {
    display: inline-flex;
    margin-left: 6px;
    vertical-align: -1px;
    color: var(--faint);
  }

  .m {
    flex: none;
    font: 10.5px var(--mono);
    color: var(--faint);
  }

  mark {
    background: none;
    color: var(--fg);
    font-weight: 600;
    box-shadow: inset 0 -2px 0 var(--line-strong);
  }

  .none {
    margin: 0;
    padding: 28px 16px;
    text-align: center;
    font: 12px var(--mono);
    color: var(--faint);
  }

  footer {
    display: flex;
    align-items: center;
    gap: 16px;
    height: 36px;
    padding: 0 16px;
    border-top: 1px solid var(--line);
    font: 11px var(--mono);
    color: var(--faint);
  }

  footer span {
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .n {
    margin-left: auto;
  }

  @media (max-width: 640px) {
    .pal {
      top: calc(10px + env(safe-area-inset-top));
    }

    .m,
    .wide,
    footer > span:not(.n) {
      display: none;
    }
  }
</style>
