<script lang="ts">
  import type { Note } from '../../notes/types'
  import { ago } from '../../ui/ago'
  import { clock } from '../../ui/now.svelte'
  import Cover from '../Cover.svelte'
  import Foot from '../Foot.svelte'

  let { note }: { note: Note } = $props()
  const t = $derived(note.title.trim())
</script>

<Cover {note} />
<h3 class:none={!t}>{t || 'Sem título'}</h3>
<div class="body">{note.body}</div>
<Foot icon="text">texto · {ago(note.updated, clock.now)}</Foot>

<style>
  h3 {
    margin: 0 0 8px;
    font-size: 14px;
    line-height: 1.3;
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .none {
    color: var(--faint);
  }

  .body {
    flex: 1;
    overflow: hidden;
    white-space: pre-line;
    overflow-wrap: anywhere;
    font-size: 12.5px;
    line-height: 1.55;
    color: var(--soft);
    mask-image: linear-gradient(to bottom, #000 60%, transparent);
  }

  :global(.small) h3 {
    font-size: 13px;
  }

  :global(.small) .body {
    font-size: 11.5px;
  }
</style>
