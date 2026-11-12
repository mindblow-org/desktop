<script module lang="ts">
  import type { Color } from '../../notes/types'

  const all: { c: Color; name: string }[] = [
    { c: '', name: 'Cinza' },
    { c: 'yellow', name: 'Amarelo' },
    { c: 'peach', name: 'Pêssego' },
    { c: 'pink', name: 'Rosa' },
    { c: 'lilac', name: 'Lavanda' },
    { c: 'sky', name: 'Céu' },
    { c: 'mint', name: 'Menta' },
  ]
</script>

<script lang="ts">
  import { store } from '../../notes/store.svelte'
  import type { Note } from '../../notes/types'

  let { note }: { note: Note } = $props()
  const on = $derived(note.color ?? '')
</script>

<div class="colors" role="radiogroup" aria-label="Cor do sticky">
  {#each all as { c, name } (c)}
    <button
      data-color={c || null}
      class:on={on === c}
      role="radio"
      aria-checked={on === c}
      aria-label={name}
      title={name}
      onclick={() => store.edit(note.id, { color: c })}
    ></button>
  {/each}
</div>

<style>
  .colors {
    display: flex;
    gap: 6px;
    margin-left: auto;
  }

  button {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 1px solid rgb(0 0 0 / 0.08);
    border-radius: 999px;
    background: var(--sticky);
    cursor: pointer;
    transition: scale 120ms var(--ease);
  }

  button:hover {
    scale: 1.12;
  }

  .on {
    box-shadow:
      0 0 0 2px var(--bg),
      0 0 0 3.5px var(--ink);
  }

  @media (prefers-color-scheme: dark) {
    button {
      border-color: rgb(255 255 255 / 0.08);
    }
  }
</style>
