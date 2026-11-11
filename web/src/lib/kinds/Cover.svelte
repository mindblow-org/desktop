<script lang="ts">
  import { src } from '../api/http'
  import type { Note } from '../notes/types'

  let { note }: { note: Note } = $props()
  const first = $derived(note.images[0])
</script>

{#if first}
  <div class="cover">
    <img src={src(first.id)} alt="" loading="lazy" decoding="async" draggable="false" />
    {#if note.images.length > 1}<span class="more">+{note.images.length - 1}</span>{/if}
  </div>
{/if}

<style>
  .cover {
    position: relative;
    flex: none;
    height: 40%;
    margin: -16px -16px 12px;
    overflow: hidden;
    border-radius: 13px 13px 0 0;
    background: var(--hover);
  }

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    pointer-events: none;
  }

  .more {
    position: absolute;
    right: 8px;
    bottom: 8px;
    height: 20px;
    padding: 0 7px;
    border-radius: 999px;
    background: var(--ink);
    color: var(--on-ink);
    font: 10px/20px var(--mono);
  }

  :global(.small) .cover {
    margin: -12px -12px 10px;
  }
</style>
