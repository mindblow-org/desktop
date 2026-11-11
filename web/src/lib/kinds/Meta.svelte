<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { Note } from '../notes/types'
  import { ago } from '../ui/ago'
  import Icon, { type Name } from '../ui/Icon.svelte'
  import { clock } from '../ui/now.svelte'

  let { note, icon, label, children }: { note: Note; icon: Name; label: string; children?: Snippet } = $props()
</script>

<div class="meta">
  <span class="k"><Icon name={icon} size={12} />{label}</span>
  <span>·</span>
  <span>editado {ago(note.updated, clock.now)}</span>
  {#if children}
    <span>·</span>
    <span>{@render children()}</span>
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
