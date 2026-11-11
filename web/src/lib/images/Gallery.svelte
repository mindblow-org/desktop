<script lang="ts">
  import { flip } from 'svelte/animate'
  import { fade } from 'svelte/transition'
  import { src } from '../api/http'
  import { store } from '../notes/store.svelte'
  import type { Note } from '../notes/types'
  import Icon from '../ui/Icon.svelte'
  import Viewer from './Viewer.svelte'

  let { note, solo = false }: { note: Note; solo?: boolean } = $props()
  let at = $state<number | null>(null)
  const ups = $derived(store.uploads.filter((u) => u.note === note.id))
  const n = $derived(note.images.length + ups.length)
  const one = $derived(n === 1 ? note.images[0] : undefined)
</script>

{#if n}
  <div class="gal" class:one={n === 1} class:two={n === 2} class:many={n >= 3} class:solo>
    {#each note.images as img, i (img.id)}
      <div
        class="tile"
        class:big={n >= 3 && i === 0}
        style:aspect-ratio={one && one.w && one.h ? `${one.w} / ${one.h}` : null}
        animate:flip={{ duration: 220 }}
        transition:fade={{ duration: 160 }}
      >
        <button class="open" aria-label="Ver imagem" onclick={() => (at = i)}>
          <img src={src(img.id)} alt="" loading="lazy" decoding="async" draggable="false" />
        </button>
        <button class="x" aria-label="Remover imagem" title="Remover imagem" onclick={() => store.detach(note.id, img.id)}>
          <Icon name="x" size={14} />
        </button>
      </div>
    {/each}
    {#each ups as u (u.key)}
      <div class="tile up" class:big={n >= 3 && !note.images.length && u === ups[0]} transition:fade={{ duration: 160 }}>
        <img src={u.url} alt="" draggable="false" /><span class="ring"></span>
      </div>
    {/each}
  </div>
{/if}

{#if at !== null}
  <Viewer {note} start={at} onclose={() => (at = null)} />
{/if}

<style>
  .gal {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-auto-rows: 132px;
    gap: 8px;
    margin: 0 0 24px;
  }

  .two {
    grid-template-columns: 1fr 1fr;
    grid-auto-rows: 220px;
  }

  .one {
    grid-template-columns: 1fr;
    grid-auto-rows: auto;
  }

  .one .tile {
    max-height: 460px;
    min-height: 160px;
  }

  .many .big {
    grid-column: span 2;
    grid-row: span 2;
  }

  .tile {
    position: relative;
    overflow: hidden;
    border-radius: var(--r-md);
    background: var(--hover);
  }

  .open {
    display: block;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    background: none;
    cursor: zoom-in;
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: scale 260ms var(--ease);
  }

  .tile:hover img {
    scale: 1.02;
  }

  .x {
    position: absolute;
    top: 6px;
    right: 6px;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: var(--paper);
    color: var(--fg);
    opacity: 0;
    cursor: pointer;
    transition: opacity 120ms;
  }

  .tile:hover .x,
  .x:focus-visible {
    opacity: 1;
  }

  .up img {
    opacity: 0.4;
    filter: grayscale(1);
  }

  .ring {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 20px;
    height: 20px;
    margin: -10px 0 0 -10px;
    border-radius: 50%;
    border: 2px solid var(--line-strong);
    border-top-color: var(--ink);
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }

  .solo.many {
    grid-template-columns: 1fr 1fr;
    grid-auto-rows: 100px;
  }

  @media (max-width: 640px) {
    .many {
      grid-template-columns: 1fr 1fr;
      grid-auto-rows: 120px;
    }

    .two {
      grid-auto-rows: 150px;
    }

    .x {
      opacity: 1;
    }
  }
</style>
