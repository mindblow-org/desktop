<script lang="ts">
  import { kinds } from '../kinds'
  import type { Note } from '../notes/types'
  import Icon from '../ui/Icon.svelte'

  let { note, x, y, tab = false }: { note: Note; x: number; y: number; tab?: boolean } = $props()
  const s = $derived(kinds[note.kind])
</script>

<div class="ghost" class:tab style:left="{x}px" style:top="{y}px">
  <span class="gt"><Icon name={s.icon} size={14} /><span>{s.title(note)}</span></span>
  <span class="gh">
    {#if tab}<Icon name="tab" size={11} />solte para abrir em aba{:else}<Icon name="detach" size={11} />solte para destacar{/if}
  </span>
</div>

<style>
  .ghost {
    position: fixed;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 220px;
    padding: 12px 14px;
    border-radius: var(--r-lg);
    background: var(--paper);
    box-shadow: var(--shadow-pop);
    pointer-events: none;
    translate: -40px -16px;
  }

  .tab {
    translate: -40px 22px;
  }

  .gt {
    display: flex;
    align-items: center;
    gap: 7px;
    overflow: hidden;
    font-size: 13px;
    white-space: nowrap;
  }

  .gt span {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .gh {
    display: flex;
    align-items: center;
    gap: 5px;
    font: 11px var(--mono);
    color: var(--faint);
  }
</style>
