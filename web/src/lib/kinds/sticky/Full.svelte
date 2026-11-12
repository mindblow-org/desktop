<script lang="ts">
  import { onMount } from 'svelte'
  import { src } from '../../api/http'
  import Viewer from '../../images/Viewer.svelte'
  import { store } from '../../notes/store.svelte'
  import type { Note } from '../../notes/types'
  import Icon from '../../ui/Icon.svelte'
  import Meta from '../Meta.svelte'
  import Colors from './Colors.svelte'

  let { note, solo = false }: { note: Note; solo?: boolean } = $props()
  let body = $derived(note.body)
  let el: HTMLTextAreaElement
  let seeing = $state(false)
  const up = $derived(store.uploads.find((u) => u.note === note.id))
  const pic = $derived(note.images[0])

  onMount(() => {
    if (!note.body && !note.images.length) el.focus()
  })
</script>

<div class="stage" class:solo>
  <div class="wrap">
    <div class="big" class:photo={pic || up} data-color={note.color || null}>
      {#if up}
        <div class="pic up"><img src={up.url} alt="" draggable="false" /><span class="ring"></span></div>
      {:else if pic}
        <div class="pic">
          <button class="open" aria-label="Ver imagem" onclick={() => (seeing = true)}>
            <img src={src(pic.id)} alt="" draggable="false" />
          </button>
          <button class="x" aria-label="Remover imagem" title="Remover imagem" onclick={() => store.detach(note.id, pic.id)}>
            <Icon name="x" size={14} />
          </button>
        </div>
      {/if}
      <textarea
        maxlength="500"
        placeholder="Escreva…"
        value={body}
        bind:this={el}
        oninput={(e) => store.edit(note.id, { body: (body = e.currentTarget.value) })}
      ></textarea>
    </div>
    {#if !solo}
      <div class="row">
        <Meta {note} icon="sticky" label="sticky">{body.length}/500</Meta>
        <Colors {note} />
      </div>
    {/if}
  </div>
</div>

{#if seeing}
  <Viewer {note} start={0} onclose={() => (seeing = false)} />
{/if}

<style>
  .stage {
    display: grid;
    place-items: center;
    min-height: 100%;
    padding: 40px 20px 80px;
  }

  .wrap {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 16px;
    max-width: min(520px, 88vw, 68vh);
    margin: 0 4px;
  }

  .big {
    display: flex;
    flex-direction: column;
    width: min(520px, 88vw, 68vh);
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: var(--r-xl);
    background: var(--sticky);
  }

  textarea {
    flex: 1;
    min-height: 0;
    padding: clamp(24px, 5vw, 48px);
    font-size: clamp(24px, 4.4vw, 46px);
    line-height: 1.12;
    font-weight: 500;
    letter-spacing: -0.035em;
    overflow: hidden;
  }

  .photo textarea {
    padding: 16px clamp(20px, 4vw, 32px) 24px;
    font-size: clamp(20px, 3vw, 30px);
    letter-spacing: -0.025em;
  }

  .pic {
    position: relative;
    flex: none;
    height: 64%;
    margin: 18px 18px 0;
    overflow: hidden;
    border-radius: 12px;
    background: var(--hover);
  }

  .open {
    display: block;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    background: none;
    cursor: zoom-in;
  }

  .pic img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .x {
    position: absolute;
    top: 8px;
    right: 8px;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: var(--paper);
    color: var(--fg);
    opacity: 0;
    cursor: pointer;
    transition: opacity 120ms;
  }

  .pic:hover .x,
  .x:focus-visible {
    opacity: 1;
  }

  .up img {
    opacity: 0.4;
    filter: grayscale(1);
  }

  .ring {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 20px;
    height: 20px;
    margin: -10px 0 0 -10px;
    border-radius: 50%;
    border: 2px solid var(--line-strong);
    border-top-color: var(--ink);
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }

  .solo {
    padding: 0;
    height: 100%;
  }

  .solo .wrap {
    width: 100%;
    height: 100%;
  }

  .solo .big {
    width: 100%;
    height: 100%;
    aspect-ratio: auto;
    border-radius: 0;
  }

  .solo textarea {
    padding: 28px;
    font-size: 26px;
    line-height: 1.18;
    letter-spacing: -0.025em;
  }

  .solo .photo textarea {
    padding: 12px 22px 20px;
    font-size: 20px;
  }

  .solo .pic {
    margin: 0 12px;
    height: 60%;
  }

  @media (max-width: 640px) {
    .x {
      opacity: 1;
    }
  }
</style>
