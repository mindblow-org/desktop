<script lang="ts">
  import { tick, type Snippet } from 'svelte'
  import { fade } from 'svelte/transition'
  import { kinds } from '../kinds'
  import { store } from '../notes/store.svelte'
  import type { Note } from '../notes/types'
  import { shell } from '../shell'
  import Icon from '../ui/Icon.svelte'
  import { uris } from './uris'

  let { note, children }: { note: Note; children: Snippet } = $props()
  let depth = $state(0)
  let count = $state(0)
  let root = $state<HTMLDivElement>()

  const has = (e: DragEvent) => [...(e.dataTransfer?.types ?? [])].includes('Files')
  const pics = (d: DataTransfer | null) => [...(d?.files ?? [])].filter((f) => f.type.startsWith('image/'))

  function enter(e: DragEvent) {
    if (!has(e)) return
    e.preventDefault()
    depth++
    count = [...(e.dataTransfer?.items ?? [])].filter((i) => i.kind === 'file').length
  }

  async function add(f: File[]) {
    void store.attach(note.id, f)
    await tick()
    root?.querySelector('.gal')?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  }

  function drop(e: DragEvent) {
    if (!has(e)) return
    e.preventDefault()
    depth = 0
    void add([...(e.dataTransfer?.files ?? [])])
  }

  async function paste(e: ClipboardEvent) {
    const f = pics(e.clipboardData)
    if (f.length) {
      e.preventDefault()
      return add(f)
    }
    const u = uris(e.clipboardData?.getData('text/uri-list') ?? '')
    if (!u.length || !shell.can.clip) return
    e.preventDefault()
    const got = await Promise.all(
      u.map(async (p) => {
        const b = await shell.clip(p.uri)
        return b && new File([b], p.name, { type: p.type })
      }),
    )
    const ok = got.filter((x) => x !== null)
    if (ok.length) await add(ok)
  }
</script>

{#if !kinds[note.kind].images}
  {@render children()}
{:else}
  <div
    class="catch"
    bind:this={root}
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
{/if}

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
