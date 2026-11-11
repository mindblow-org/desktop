<script lang="ts">
  import { store } from './lib/notes/store.svelte'
  import Main from './view/Main.svelte'
  import Solo from './view/Solo.svelte'

  const solo = location.pathname.match(/^\/n\/([\w-]+)/)?.[1]

  store.start()
  addEventListener('pagehide', () => store.flushAll())
</script>

{#if solo}
  <Solo id={solo} />
{:else}
  <Main />
{/if}
