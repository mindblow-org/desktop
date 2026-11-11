<script lang="ts">
  import type { Note } from '../../notes/types'
  import Icon from '../../ui/Icon.svelte'
  import Cover from '../Cover.svelte'
  import Foot from '../Foot.svelte'

  let { note }: { note: Note } = $props()
  const done = $derived(note.items.filter((i) => i.done).length)
  const t = $derived(note.title.trim())
</script>

<Cover {note} />
<h3 class:none={!t}>{t || 'Lista'}</h3>
<ul>
  {#each note.items.slice(0, 12) as i (i.id)}
    <li class:done={i.done}>
      <span class="box" class:done={i.done}><Icon name="check" size={9} stroke={3} /></span>
      <span class="t">{i.text}</span>
    </li>
  {/each}
</ul>
<Foot icon="todo">
  {done}/{note.items.length}
  <span class="progress"><b style:width="{note.items.length ? (done / note.items.length) * 100 : 0}%"></b></span>
</Foot>

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

  ul {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 7px;
    margin: 0;
    padding: 0;
    list-style: none;
    overflow: hidden;
    font-size: 12.5px;
    color: var(--soft);
    mask-image: linear-gradient(to bottom, #000 70%, transparent);
  }

  li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    line-height: 1.35;
  }

  li .box {
    margin-top: 1px;
  }

  .t {
    overflow-wrap: anywhere;
  }

  li.done .t {
    color: var(--faint);
    text-decoration: line-through;
  }

  :global(.small) h3 {
    font-size: 13px;
  }

  :global(.small) ul {
    font-size: 11.5px;
    gap: 6px;
  }
</style>
