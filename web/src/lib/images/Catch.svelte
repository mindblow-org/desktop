<script lang="ts">
  import type { Snippet } from 'svelte'
  import { fade } from 'svelte/transition'
  import { store } from '../notes/store.svelte'
  import type { Note } from '../notes/types'
  import Icon from '../ui/Icon.svelte'

  let { note, children }: { note: Note; children: Snippet } = $props()
  let depth = $state(0)
  let count = $state(0)

  const has = (e: DragEvent) => [...(e.dataTransfer?.types ?? [])].includes('Files')
  const pics = (d: DataTransfer | null) => [...(d?.files ?? [])].filter((f) => f.type.startsWith('image/'))

  function enter(e: DragEvent) {
    if (!has(e)) return
    e.preventDefault()
    depth++
    count = [...(e.dataTransfer?.items ?? [])].filter((i) => i.kind === 'file').length
  }

  function drop(e: DragEvent) {
    if (!has(e)) return
    e.preventDefault()
    depth = 0
    void store.attach(note.id, [...(e.dataTransfer?.files ?? [])])
  }

  function paste(e: ClipboardEvent) {
    const f = pics(e.clipboardData)
    if (!f.length) return
    e.preventDefault()
    void store.attach(note.id, f)
  }
</script>

<div
  class="catch"
  role="region"
  aria-label="Nota"
  onpaste={paste}
  ondragenter={enter}
  ondragover={(e) => has(e) && e.preventDefault()}
  ondragleave={() => (depth = Math.max(0, depth - 1))}
  ondrop={drop}
>
  {@render children()}
  {#if depth}
    <div class="drop" transition:fade={{ duration: 120 }}>
      <div class="zone">
        <Icon name="image" size={28} />
        <b>Solte para anexar</b>
        <span>
          {#if note.kind === 'sticky'}1 imagem · troca a atual{:else}{count || 1} {count > 1 ? 'imagens' : 'imagem'} · reduzidas para até 2048 px{/if}
        </span>
      </div>
    </div>
  {/if}
</div>

<style>
  .catch {
    min-height: 100%;
  }

  .drop {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--bg) 82%, transparent);
    pointer-events: none;
  }

  .zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 48px 72px;
    border: 2px dashed var(--ink);
    border-radius: var(--r-xl);
    background: var(--paper);
    box-shadow: var(--shadow-pop);
  }

  b {
    font-size: 17px;
    font-weight: 600;
  }

  span {
    font: 11.5px var(--mono);
    color: var(--muted);
  }
</style>
