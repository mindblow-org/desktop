<script lang="ts">
  import { onMount } from 'svelte'
  import { fade } from 'svelte/transition'
  import { src } from '../api/http'
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Note } from '../notes/types'
  import Icon from '../ui/Icon.svelte'
  import { portal } from '../ui/portal'

  let { note, start, onclose }: { note: Note; start: number; onclose: () => void } = $props()
  let i = $derived(start)
  const imgs = $derived(note.images)
  const at = $derived(Math.min(i, imgs.length - 1))
  const img = $derived(imgs[at])
  const kb = $derived(img ? Math.max(1, Math.round(img.size / 1024)) : 0)

  $effect(() => {
    if (!imgs.length) onclose()
  })

  onMount(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onclose()
      else if (e.key === 'ArrowRight') step(1)
      else if (e.key === 'ArrowLeft') step(-1)
      else return
      e.preventDefault()
      e.stopPropagation()
    }
    addEventListener('keydown', key, true)
    return () => removeEventListener('keydown', key, true)
  })

  function step(d: number) {
    if (imgs.length) i = (at + d + imgs.length) % imgs.length
  }

  function del() {
    if (!img) return
    store.detach(note.id, img.id)
    if (at > 0 && at >= imgs.length - 1) i = at - 1
  }
</script>

{#if img}
  <div class="view" role="dialog" aria-label="Imagem" transition:fade={{ duration: 160 }} {@attach portal}>
    <header>
      <Icon name={kinds[note.kind].icon} size={14} />
      <b>{kinds[note.kind].title(note)}</b>
      <span>·</span><span>{at + 1} de {imgs.length}</span>
      {#if img.w}<span class="wide">·</span><span class="wide">{img.w} × {img.h}</span>{/if}
      <span class="wide">·</span><span class="wide">{kb >= 1024 ? `${(kb / 1024).toFixed(1)} MB` : `${kb} KB`}</span>
      <span class="sp"></span>
      <button class="b" title="Remover imagem" aria-label="Remover imagem" onclick={del}><Icon name="trash" /></button>
      <button class="b" title="Fechar  Esc" aria-label="Fechar" onclick={onclose}><Icon name="x" /></button>
    </header>
    <div class="stage" role="presentation" onclick={(e) => e.target === e.currentTarget && onclose()}>
      {#if imgs.length > 1}
        <button class="nav l" aria-label="Anterior" onclick={() => step(-1)}><Icon name="back" size={20} /></button>
      {/if}
      {#key img.id}
        <img src={src(img.id)} alt="" in:fade={{ duration: 180 }} draggable="false" />
      {/key}
      {#if imgs.length > 1}
        <button class="nav r" aria-label="Próxima" onclick={() => step(1)}><Icon name="next" size={20} /></button>
      {/if}
    </div>
    {#if imgs.length > 1}
      <div class="strip">
        {#each imgs as m, k (m.id)}
          <button class:on={k === at} aria-label="Imagem {k + 1}" onclick={() => (i = k)}><img src={src(m.id)} alt="" draggable="false" /></button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .view {
    --v-bg: #0e0e0e;
    --v-panel: #181818;
    --v-line: #2a2a2a;
    --v-fg: #ededeb;
    --v-muted: #8a8a85;
    position: fixed;
    inset: 0;
    z-index: 80;
    display: grid;
    grid-template-rows: 56px 1fr auto;
    background: color-mix(in srgb, var(--v-bg) 98%, transparent);
    color: var(--v-fg);
  }

  header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 14px 0 20px;
    padding-top: env(safe-area-inset-top);
    font: 12px var(--mono);
    color: var(--v-muted);
    white-space: nowrap;
  }

  header b {
    overflow: hidden;
    text-overflow: ellipsis;
    font: 500 13px var(--sans);
    color: var(--v-fg);
  }

  .sp {
    flex: 1;
  }

  .b {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 0;
    border-radius: var(--r-md);
    background: none;
    color: var(--v-muted);
    cursor: pointer;
  }

  .b:hover {
    background: var(--v-panel);
    color: var(--v-fg);
  }

  .stage {
    position: relative;
    display: grid;
    place-items: center;
    min-height: 0;
    padding: 0 96px 16px;
  }

  .stage img {
    max-width: 100%;
    max-height: 100%;
    border-radius: var(--r-md);
    box-shadow: 0 22px 48px rgb(0 0 0 / 0.6);
  }

  .nav {
    position: absolute;
    top: 50%;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    margin-top: -22px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: var(--v-panel);
    box-shadow: 0 0 0 1px var(--v-line);
    color: var(--v-fg);
    cursor: pointer;
  }

  .l {
    left: 28px;
  }

  .r {
    right: 28px;
  }

  .strip {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 12px 16px calc(20px + env(safe-area-inset-bottom));
    overflow-x: auto;
  }

  .strip button {
    flex: none;
    width: 72px;
    height: 52px;
    padding: 0;
    overflow: hidden;
    border: 0;
    border-radius: var(--r-sm);
    background: var(--v-panel);
    opacity: 0.45;
    cursor: pointer;
    transition: opacity 160ms;
  }

  .strip button.on {
    opacity: 1;
    box-shadow: 0 0 0 2px var(--v-fg);
  }

  .strip img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  @media (max-width: 640px) {
    .stage {
      padding: 0 8px 12px;
    }

    .nav,
    .wide {
      display: none;
    }
  }
</style>
