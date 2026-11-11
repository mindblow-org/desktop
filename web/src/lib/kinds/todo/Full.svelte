<script lang="ts">
  import { onMount } from 'svelte'
  import { flip } from 'svelte/animate'
  import { crossfade, fade } from 'svelte/transition'
  import { uid } from '../../local'
  import { store } from '../../notes/store.svelte'
  import type { Item, Note } from '../../notes/types'
  import Gallery from '../../images/Gallery.svelte'
  import { fit } from '../../ui/grow'
  import Icon from '../../ui/Icon.svelte'
  import Meta from '../Meta.svelte'

  let { note, solo = false }: { note: Note; solo?: boolean } = $props()
  let title = $derived(note.title)
  let draft = $state('')
  let folded = $state(false)
  let te: HTMLTextAreaElement
  let ne: HTMLInputElement
  const open = $derived(note.items.filter((i) => !i.done))
  const done = $derived(note.items.filter((i) => i.done))
  const pct = $derived(note.items.length ? (done.length / note.items.length) * 100 : 0)
  const [send, receive] = crossfade({ duration: 220, fallback: (n) => fade(n, { duration: 160 }) })

  onMount(() => {
    if (!note.title && !note.items.length) te.focus()
  })

  function add() {
    const text = draft.trim()
    if (!text) return
    store.push(note.id, { id: uid(), text, done: false })
    draft = ''
  }

  function rename(i: Item, v: string) {
    const text = v.trim()
    if (!text) store.pull(note.id, i.id)
    else if (text !== i.text) store.mark(note.id, i.id, { text })
  }

  function keys(e: KeyboardEvent, i: Item) {
    const el = e.currentTarget as HTMLInputElement
    if (e.key === 'Enter') {
      e.preventDefault()
      ne.focus()
    } else if (e.key === 'Backspace' && !el.value) {
      e.preventDefault()
      store.pull(note.id, i.id)
      ne.focus()
    }
  }
</script>

<article class="page" class:solo>
  {#if !solo}
    <Meta {note} icon="todo" label="todo" />
  {/if}
  <textarea
    class="title"
    rows="1"
    maxlength="200"
    placeholder="Lista"
    value={title}
    bind:this={te}
    {@attach fit}
    oninput={(e) => store.edit(note.id, { title: (title = e.currentTarget.value) })}
    onkeydown={(e) => {
      if (e.key === 'Enter') {
        e.preventDefault()
        ne.focus()
      }
    }}
  ></textarea>

  {#if note.items.length}
    <div class="prog"><span>{done.length} de {note.items.length}</span><span class="progress"><b style:width="{pct}%"></b></span></div>
  {/if}

  <ul class="rows">
    {#each open as i (i.id)}
      <li class="row" class:done={i.done} animate:flip={{ duration: 220 }} in:receive={{ key: i.id }} out:send={{ key: i.id }}>
        <button class="box" class:done={i.done} aria-label={i.done ? 'Desmarcar' : 'Marcar'} onclick={() => store.mark(note.id, i.id, { done: !i.done })}>
          <Icon name="check" size={12} stroke={2.6} />
        </button>
        <input class="t" value={i.text} maxlength="500" onchange={(e) => rename(i, e.currentTarget.value)} onkeydown={(e) => keys(e, i)} />
        <button class="del" aria-label="Remover item" onclick={() => store.pull(note.id, i.id)}><Icon name="x" size={14} /></button>
      </li>
    {/each}
  </ul>
  <div class="row new">
    <span class="plus"><Icon name="plus" size={14} /></span>
    <input
      placeholder="Adicionar item"
      maxlength="500"
      bind:value={draft}
      bind:this={ne}
      onkeydown={(e) => {
        if (e.key === 'Enter') add()
      }}
    />
    {#if draft}<span class="kbd">enter</span>{/if}
  </div>

  <div class="pics"><Gallery {note} {solo} /></div>

  {#if done.length}
    <button class="sec" class:folded onclick={() => (folded = !folded)}><Icon name="chev" size={13} />Feitos · {done.length}</button>
    {#if !folded}
      <ul class="rows">
        {#each done as i (i.id)}
          <li class="row" class:done={i.done} animate:flip={{ duration: 220 }} in:receive={{ key: i.id }} out:send={{ key: i.id }}>
            <button class="box" class:done={i.done} aria-label={i.done ? 'Desmarcar' : 'Marcar'} onclick={() => store.mark(note.id, i.id, { done: !i.done })}>
              <Icon name="check" size={12} stroke={2.6} />
            </button>
            <input class="t" value={i.text} maxlength="500" onchange={(e) => rename(i, e.currentTarget.value)} onkeydown={(e) => keys(e, i)} />
            <button class="del" aria-label="Remover item" onclick={() => store.pull(note.id, i.id)}><Icon name="x" size={14} /></button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
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
    margin: 14px 0 16px;
    font-size: 34px;
    line-height: 1.15;
    font-weight: 600;
    letter-spacing: -0.025em;
    overflow: hidden;
  }

  .prog {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 24px;
    font: 11.5px var(--mono);
    color: var(--muted);
  }

  .prog .progress {
    height: 4px;
  }

  .pics {
    margin-top: 24px;
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 42px;
    margin: 0 -10px;
    padding: 0 10px;
    border-radius: var(--r-md);
    font-size: 15.5px;
    background: var(--bg);
    transition: background 120ms;
  }

  .row:hover {
    background: var(--hover);
  }

  .row .box {
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 5px;
    background: none;
    cursor: pointer;
  }

  .row .box.done {
    background: var(--ink);
  }

  .t {
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  .row.done .t {
    color: var(--faint);
    text-decoration: line-through;
  }

  .del {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: none;
    color: var(--faint);
    opacity: 0;
    cursor: pointer;
    transition: opacity 120ms;
  }

  .row:hover .del,
  .del:focus-visible {
    opacity: 1;
  }

  .new {
    margin-top: 6px;
    background: var(--paper);
    box-shadow: inset 0 0 0 1px var(--line);
    color: var(--faint);
  }

  .new:hover {
    background: var(--paper);
  }

  .new input {
    flex: 1;
    min-width: 0;
    height: 100%;
    color: var(--fg);
  }

  .plus {
    display: grid;
    place-items: center;
    width: 18px;
  }

  .sec {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 32px 0 8px;
    padding: 0;
    border: 0;
    background: none;
    font: 500 11px var(--mono);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--faint);
    cursor: pointer;
  }

  .sec :global(svg) {
    transition: rotate 160ms var(--ease);
  }

  .sec.folded :global(svg) {
    rotate: -90deg;
  }

  .solo {
    padding: 18px 22px 40px;
  }

  .solo .title {
    margin-top: 0;
    font-size: 22px;
  }

  .solo .row {
    height: 38px;
    font-size: 14.5px;
  }

  @media (max-width: 640px) {
    .page {
      padding: 12px 20px 80px;
    }

    .title {
      font-size: 28px;
    }

    .row {
      height: 50px;
      font-size: 17px;
    }

    .row .box {
      width: 22px;
      height: 22px;
    }

    .del {
      opacity: 1;
    }
  }
</style>
