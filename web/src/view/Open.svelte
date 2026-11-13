<script lang="ts">
  import { onMount } from 'svelte'
  import Catch from '../lib/images/Catch.svelte'
  import { kinds } from '../lib/kinds'
  import Meta from '../lib/kinds/Meta.svelte'
  import { store } from '../lib/notes/store.svelte'
  import type { Note } from '../lib/notes/types'
  import { shell } from '../lib/shell'
  import Icon from '../lib/ui/Icon.svelte'

  let { note, onback, ondetach }: { note: Note; onback: () => void; ondetach: () => void } = $props()
  const s = $derived(kinds[note.kind])
  let sure = $state(false)
  let kit = $state(false)
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

  // O voltar (Esc) fecha a barra do board antes de sair dele, depois do que o board já tratou (seleção, ferramenta).
  onMount(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || !kit || e.cancelBubble) return
      e.stopPropagation()
      kit = false
    }
    addEventListener('keydown', key, true)
    return () => removeEventListener('keydown', key, true)
  })
</script>

<section class="open" class:board={note.kind === 'board'} style:view-transition-name="n-{note.id}">
  <button class="ib back" aria-label="Voltar às notas" title="Voltar às notas" onclick={onback}><Icon name="back" /></button>
  <div class="body">
    <Catch {note}>
      <s.Full {note} {kit} />
    </Catch>
  </div>
  {#if s.foot}
    <div class="foot"><Meta {note} icon={s.icon} label={s.label.toLowerCase()} extra={s.foot(note)} /></div>
  {/if}
  <div class="tools" role="toolbar" aria-label="Ações da nota">
    {#if note.kind === 'board'}
      <button class="ib kit" class:on={kit} title="Ferramentas" aria-label="Ferramentas do board" aria-pressed={kit} onclick={() => (kit = !kit)}
        ><Icon name="pen" /></button
      >
    {/if}
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
  {#if store.notice}
    <footer>{store.notice}</footer>
  {/if}
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
    right: 20px;
    z-index: 2;
    display: flex;
    gap: 2px;
  }

  .back {
    display: none;
  }

  .kit {
    display: none;
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

  .foot {
    width: 100%;
    max-width: 728px;
    margin: 0 auto;
    padding: 10px 24px 16px;
  }

  footer {
    position: absolute;
    left: 20px;
    right: 20px;
    bottom: 16px;
    text-align: right;
    font: 11px var(--mono);
    color: var(--fg);
    pointer-events: none;
  }

  /* No celular as ações viram a toolbar flutuante de baixo, como a do grid, com o voltar à esquerda dela. No board ela ganha o botão que abre a barra de ferramentas dele, logo acima. */
  @media (max-width: 640px) {
    /* Voltar é um botão redondo no canto de baixo, à esquerda da toolbar, no mesmo estilo dela. */
    .back {
      position: absolute;
      left: 16px;
      bottom: calc(16px + env(safe-area-inset-bottom));
      z-index: 2;
      display: grid;
      width: 48px;
      height: 48px;
      border-radius: 999px;
      background: var(--ink);
      box-shadow: var(--shadow-pop);
      color: var(--on-ink);
    }

    .back:hover {
      background: var(--ink);
      color: var(--on-ink);
    }

    .back :global(svg) {
      width: 20px;
      height: 20px;
    }

    .open:not(.board) .body {
      padding-top: 14px;
      padding-bottom: calc(var(--dock) + 16px + env(safe-area-inset-bottom));
    }

    .tools {
      top: auto;
      left: 0;
      right: 0;
      bottom: calc(16px + env(safe-area-inset-bottom));
      align-items: center;
      width: fit-content;
      height: 48px;
      margin: 0 auto;
      padding: 0 4px;
      border-radius: 999px;
      background: var(--ink);
      box-shadow: var(--shadow-pop);
    }

    .tools .ib {
      min-width: 48px;
      height: 40px;
      border-radius: 999px;
      color: var(--on-ink);
    }

    .tools .ib:hover {
      background: color-mix(in srgb, var(--on-ink) 16%, transparent);
      color: var(--on-ink);
    }

    .tools .ib.on,
    .tools .sure,
    .tools .sure:hover {
      background: var(--on-ink);
      color: var(--ink);
    }

    .tools .kit {
      display: grid;
    }

    .tools .ib :global(svg) {
      width: 19px;
      height: 19px;
    }

    footer {
      bottom: calc(var(--dock) + 10px + env(safe-area-inset-bottom));
    }

    .open:has(> .foot) .body {
      padding-bottom: 0;
    }

    .foot {
      padding: 8px 20px calc(var(--dock) + 16px + env(safe-area-inset-bottom));
    }

    .open:has(> footer) .foot {
      visibility: hidden;
    }
  }
</style>
