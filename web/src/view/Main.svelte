<script lang="ts">
  import Palette from '../lib/find/Palette.svelte'
  import Grid from '../lib/grid/Grid.svelte'
  import { kinds } from '../lib/kinds'
  import { store } from '../lib/notes/store.svelte'
  import type { Kind } from '../lib/notes/types'
  import { shell } from '../lib/shell'
  import Bar from '../lib/tabs/Bar.svelte'
  import { tabs } from '../lib/tabs/store.svelte'
  import { go } from '../lib/ui/motion.svelte'
  import Lock from './Lock.svelte'
  import Open from './Open.svelte'

  let picking = $state(false)
  let finding = $state(false)
  const note = $derived(tabs.on ? store.find(tabs.on) : undefined)

  $effect(() => {
    if (store.ready) tabs.prune(new Set(store.list.map((n) => n.id)))
  })

  shell.onhome((id) => open(id))

  function open(id: string) {
    go(() => tabs.open(id), id)
  }

  function tidy(id: string | null) {
    const n = id ? store.find(id) : undefined
    if (n && kinds[n.kind].empty(n)) store.remove(n.id)
  }

  function home() {
    const id = tabs.on
    tidy(id)
    go(() => tabs.home(), id)
  }

  function close(id: string) {
    tidy(id)
    go(() => tabs.close(id))
  }

  function detach() {
    if (!note) return
    const n = note
    void store.flush(n.id)
    shell.detach(n)
    go(() => tabs.close(n.id))
  }

  async function pick(k: Kind) {
    const n = await store.add(k)
    go(() => tabs.open(n.id))
  }

  function typing(t: EventTarget | null) {
    return t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA)$/.test(t.tagName))
  }

  function found(id: string) {
    finding = false
    open(id)
  }

  // #new=<tipo> cria e abre uma nota; #open=<id> abre uma. Atalhos do PWA e do app Android.
  function hash() {
    const p = new URLSearchParams(location.hash.slice(1))
    const k = p.get('new')
    const id = p.get('open')
    if (!k && !id) return
    history.replaceState(null, '', location.pathname + location.search)
    if (k && k in kinds) void pick(k as Kind)
    else if (id) open(id)
  }

  // Botão voltar do app Android: faz o papel do Esc e diz se havia algo para fechar.
  function back(): boolean {
    if (finding) {
      finding = false
      return true
    }
    if (!tabs.on && !document.querySelector('[role="dialog"]')) return false
    document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    return true
  }

  hash()
  ;(window as unknown as { mindblowBack: () => boolean }).mindblowBack = back

  function key(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault()
      picking = false
      finding = !finding
    } else if (finding) {
      return
    } else if (e.key === 'Escape' && !picking) {
      if (typing(e.target)) return (e.target as HTMLElement).blur()
      if (tabs.on) home()
    } else if (e.key.toLowerCase() === 'd' && e.ctrlKey && e.shiftKey && shell.can.detach) {
      e.preventDefault()
      detach()
    } else if (e.key.toLowerCase() === 'w' && e.ctrlKey && shell.can.pin && tabs.on) {
      e.preventDefault()
      close(tabs.on)
    } else if (e.key === 'n' && !e.ctrlKey && !e.metaKey && !e.altKey && !typing(e.target)) {
      e.preventDefault()
      picking = true
    }
  }
</script>

<svelte:window onkeydown={key} onhashchange={hash} />

<div class="app">
  <Bar bind:picking onopen={open} onclose={close} onhome={home} onpick={pick} onfind={() => (finding = true)} />
  <main>
    <div class="pane" hidden={!!note}>
      <Grid onopen={open} />
    </div>
    {#if note}
      {#key note.id}
        <Open {note} onback={home} ondetach={detach} />
      {/key}
    {/if}
  </main>
</div>

{#if finding}
  <Palette onopen={found} onclose={() => (finding = false)} />
{/if}

{#if store.locked}
  <Lock />
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  main {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .pane {
    position: absolute;
    inset: 0;
    overflow: auto;
  }

  /* Sem a barra de cima no celular, o recuo da barra de status passa para cá. */
  @media (max-width: 640px) {
    .app {
      padding-top: env(safe-area-inset-top);
    }
  }
</style>
