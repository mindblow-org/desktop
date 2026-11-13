<script lang="ts">
  import type { Note } from '../notes/types'
  import { ago } from '../ui/ago'
  import Icon, { type Name } from '../ui/Icon.svelte'
  import { clock } from '../ui/now.svelte'

  let { note, icon, label, extra }: { note: Note; icon: Name; label: string; extra?: string } = $props()
</script>

<div class="meta">
  <span class="k"><Icon name={icon} size={12} />{label}</span>
  <span>·</span>
  <span>editado {ago(note.updated, clock.now)}</span>
  {#if extra}
    <span>·</span>
    <span>{extra}</span>
  {/if}
</div>

<style>
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font: 11.5px var(--mono);
    color: var(--faint);
    white-space: nowrap;
  }

  .k {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--muted);
  }
</style>
