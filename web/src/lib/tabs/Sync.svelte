<script lang="ts">
  import { store } from '../notes/store.svelte'

  const label = $derived(
    store.online ? `${store.peers} ${store.peers === 1 ? 'dispositivo' : 'dispositivos'}` : 'offline',
  )
</script>

<div class="sync" class:off={!store.online} title={store.online ? 'sincronizado ao vivo' : 'reconectando…'}>
  <i></i><span>{label}</span>
</div>

<style>
  .sync {
    display: flex;
    align-items: center;
    gap: 7px;
    font: 11.5px var(--mono);
    color: var(--muted);
    white-space: nowrap;
  }

  i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ink);
    transition:
      background 200ms,
      box-shadow 200ms;
  }

  .off i {
    background: transparent;
    box-shadow: inset 0 0 0 1.5px var(--muted);
  }

  @media (max-width: 640px) {
    span {
      display: none;
    }
  }
</style>
