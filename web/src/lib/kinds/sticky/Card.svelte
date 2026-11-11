<script lang="ts">
  import { src } from '../../api/http'
  import type { Note } from '../../notes/types'

  let { note }: { note: Note } = $props()
  const v = $derived(note.body.trim())
  const pic = $derived(note.images[0])
</script>

{#if pic}
  <img src={src(pic.id)} alt="" loading="lazy" decoding="async" draggable="false" />
{/if}
<p class:none={!v && !pic} class:photo={!!pic}>{v || (pic ? '' : 'vazio')}</p>

<style>
  p {
    margin: 0;
    font-size: 17px;
    line-height: 1.32;
    font-weight: 500;
    letter-spacing: -0.015em;
    white-space: pre-line;
    overflow-wrap: anywhere;
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 7;
    line-clamp: 7;
  }

  .none {
    color: var(--faint);
  }

  img {
    display: block;
    flex: none;
    width: calc(100% + 16px);
    height: 66%;
    margin: -8px -8px 10px;
    object-fit: cover;
    border-radius: 6px;
    pointer-events: none;
  }

  .photo {
    padding: 0 2px;
    font-size: 14px;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  :global(.small) p {
    font-size: 15px;
  }
</style>
