<script lang="ts">
  import Catch from '../lib/images/Catch.svelte'
  import { kinds } from '../lib/kinds'
  import { store } from '../lib/notes/store.svelte'
  import { shell } from '../lib/shell'
  import Icon from '../lib/ui/Icon.svelte'
  import Lock from './Lock.svelte'

  let { id }: { id: string } = $props()
  let pinned = $state(false)
  const note = $derived(store.find(id))
  const s = $derived(note ? kinds[note.kind] : undefined)

  $effect(() => {
    if (note && s) document.title = s.title(note)
  })

  async function pin() {
    pinned = !pinned
    await shell.pin(pinned)
  }
</script>

<div class="solo {note?.kind ?? ''}">
  <header>
    {#if s && note}
      <span class="k"><Icon name={s.icon} size={14} /></span>
      <span class="t">{note.kind === 'sticky' ? '' : s.title(note)}</span>
    {/if}
    <span class="sp"></span>
    {#if shell.can.pin}
      <button class="ib" class:on={pinned} title={pinned ? 'Fixado no topo' : 'Fixar no topo'} aria-label="Fixar no topo" onclick={pin}><Icon name="pin" size={14} /></button>
    {/if}
    <button class="ib" title="Voltar para o app" aria-label="Voltar para o app" onclick={() => shell.home(id)}><Icon name="home" size={14} /></button>
  </header>
  <div class="body">
    {#if note && s}
      <Catch {note}>
        <s.Full {note} solo />
      </Catch>
    {:else if store.ready}
      <p class="gone">esta nota foi excluída</p>
    {/if}
  </div>
</div>

{#if store.locked}
  <Lock />
{/if}

<style>
  .solo {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--paper);
  }

  .solo.sticky {
    background: var(--sticky);
  }

  header {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
    padding: 0 6px 0 14px;
    flex: none;
    font-size: 13px;
    font-weight: 500;
    border-bottom: 1px solid var(--line);
  }

  .sticky header {
    border-bottom: 0;
  }

  header .ib {
    min-width: 28px;
    height: 28px;
  }

  .k {
    display: flex;
    color: var(--muted);
  }

  .t {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sp {
    flex: 1;
  }

  .body {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .gone {
    margin: 30vh 0 0;
    text-align: center;
    font: 12px var(--mono);
    color: var(--faint);
  }
</style>
