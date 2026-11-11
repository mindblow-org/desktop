<script lang="ts">
  import { onMount } from 'svelte'
  import { store } from '../../notes/store.svelte'
  import type { Note } from '../../notes/types'
  import Gallery from '../../images/Gallery.svelte'
  import { fit } from '../../ui/grow'
  import Meta from '../Meta.svelte'

  let { note, solo = false }: { note: Note; solo?: boolean } = $props()
  let title = $derived(note.title)
  let body = $derived(note.body)
  let te: HTMLTextAreaElement
  let be: HTMLTextAreaElement
  const words = $derived(body.trim() ? body.trim().split(/\s+/).length : 0)

  onMount(() => {
    if (!note.title && !note.body) te.focus()
  })
</script>

<article class="page" class:solo>
  {#if !solo}
    <Meta {note} icon="text" label="texto">{words} {words === 1 ? 'palavra' : 'palavras'}</Meta>
  {/if}
  <textarea
    class="title"
    rows="1"
    maxlength="200"
    placeholder="Título"
    value={title}
    bind:this={te}
    {@attach fit}
    oninput={(e) => store.edit(note.id, { title: (title = e.currentTarget.value) })}
    onkeydown={(e) => {
      if (e.key === 'Enter') {
        e.preventDefault()
        be.focus()
      }
    }}
  ></textarea>
  <Gallery {note} {solo} />
  <textarea
    class="body"
    rows="6"
    placeholder="Escreva…"
    value={body}
    bind:this={be}
    {@attach fit}
    oninput={(e) => store.edit(note.id, { body: (body = e.currentTarget.value) })}
  ></textarea>
</article>

<style>
  .page {
    max-width: 728px;
    margin: 0 auto;
    padding: 64px 24px 120px;
  }

  .title {
    display: block;
    width: 100%;
    margin: 14px 0 24px;
    font-size: 34px;
    line-height: 1.15;
    font-weight: 600;
    letter-spacing: -0.025em;
    overflow: hidden;
  }

  .body {
    display: block;
    width: 100%;
    min-height: 40vh;
    font-size: 16.5px;
    line-height: 1.75;
    color: var(--soft);
    overflow: hidden;
  }

  .solo {
    padding: 20px 22px 40px;
  }

  .solo .title {
    margin-top: 0;
    font-size: 24px;
  }

  .solo .body {
    font-size: 15px;
    min-height: 60vh;
  }

  @media (max-width: 640px) {
    .page {
      padding: 12px 20px 80px;
    }

    .title {
      font-size: 28px;
    }
  }
</style>
