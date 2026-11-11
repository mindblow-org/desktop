<script lang="ts">
  import Catch from '../lib/images/Catch.svelte'
  import { kinds } from '../lib/kinds'
  import { store } from '../lib/notes/store.svelte'
  import type { Note } from '../lib/notes/types'
  import { shell } from '../lib/shell'
  import Icon from '../lib/ui/Icon.svelte'

  let { note, onback, ondetach }: { note: Note; onback: () => void; ondetach: () => void } = $props()
  const s = $derived(kinds[note.kind])
  let sure = $state(false)
  let file = $state<HTMLInputElement>()
  let t: ReturnType<typeof setTimeout> | undefined

  function del() {
    if (!sure) {
      sure = true
      t = setTimeout(() => (sure = false), 2500)
      return
    }
    clearTimeout(t)
    store.remove(note.id)
    onback()
  }
</script>

<section class="open" style:view-transition-name="n-{note.id}">
  <div class="tools">
    <button class="ib back" onclick={onback}><Icon name="back" size={18} />Notas</button>
    <span class="sp"></span>
    {#if s.images}
      <input
        type="file"
        accept="image/*"
        multiple={s.images > 1}
        hidden
        bind:this={file}
        onchange={(e) => {
          void store.attach(note.id, [...(e.currentTarget.files ?? [])])
          e.currentTarget.value = ''
        }}
      />
      <button
        class="ib"
        title={note.kind === 'sticky' && note.images.length ? 'Trocar imagem' : 'Anexar imagem (ou cole com Ctrl+V)'}
        aria-label="Anexar imagem"
        onclick={() => file?.click()}><Icon name="image" /></button
      >
    {/if}
    <button
      class="ib"
      class:on={note.pinned}
      title={note.pinned ? 'Desafixar' : 'Fixar no topo do grid'}
      aria-label={note.pinned ? 'Desafixar' : 'Fixar'}
      onclick={() => store.pin(note.id, !note.pinned)}><Icon name="pin" /></button
    >
    {#if shell.can.detach}
      <button class="ib" title="Destacar em janela  Ctrl+Shift+D" aria-label="Destacar em janela" onclick={ondetach}><Icon name="detach" /></button>
    {/if}
    <button class="ib" class:sure title={sure ? 'Clique de novo para excluir' : 'Excluir'} aria-label="Excluir" onclick={del}>
      {#if sure}<span class="ask">excluir?</span>{:else}<Icon name="trash" />{/if}
    </button>
  </div>
  <div class="body">
    <Catch {note}>
      <s.Full {note} />
    </Catch>
  </div>
  <footer>
    <span class="esc"><span class="kbd">esc</span>volta ao grid</span>
    {#if store.notice}
      <span class="state notice">{store.notice}</span>
    {:else}
      <span class="state" class:off={!store.online}
        ><i></i>{store.uploads.length ? 'enviando imagem…' : store.busy ? 'salvando…' : store.online ? 'salvo · sincronizado' : 'offline'}</span
      >
    {/if}
  </footer>
</section>

<style>
  .open {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }

  .tools {
    position: absolute;
    top: 16px;
    left: 20px;
    right: 20px;
    z-index: 2;
    display: flex;
    gap: 2px;
    pointer-events: none;
  }

  .tools > * {
    pointer-events: auto;
  }

  .sp {
    flex: 1;
  }

  .back {
    display: none;
    gap: 2px;
    font-size: 15px;
    padding: 0 8px 0 2px;
  }

  .sure {
    background: var(--ink);
    color: var(--on-ink);
  }

  .sure:hover {
    background: var(--ink);
    color: var(--on-ink);
  }

  .ask {
    font: 500 11.5px var(--mono);
    padding: 0 4px;
  }

  .body {
    position: relative;
    flex: 1;
    overflow: auto;
  }

  footer {
    position: absolute;
    left: 20px;
    right: 20px;
    bottom: 16px;
    display: flex;
    justify-content: space-between;
    font: 11px var(--mono);
    color: var(--faint);
    pointer-events: none;
  }

  footer span {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .state i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ink);
  }

  .notice {
    color: var(--fg);
  }

  .state.off i {
    background: none;
    box-shadow: inset 0 0 0 1.5px var(--muted);
  }

  @media (max-width: 640px) {
    .tools {
      position: static;
      padding: 6px 10px 0;
    }

    .back {
      display: flex;
    }

    .esc {
      visibility: hidden;
    }

    footer {
      bottom: calc(10px + env(safe-area-inset-bottom));
    }
  }
</style>
