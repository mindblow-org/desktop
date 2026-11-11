<script lang="ts">
  import { token } from '../lib/api/token'
  import { store } from '../lib/notes/store.svelte'

  let v = $state(token.get())

  function save(e: SubmitEvent) {
    e.preventDefault()
    token.set(v.trim())
    void store.load()
  }
</script>

<div class="lock">
  <form onsubmit={save}>
    <span class="mark"></span>
    <p>Este servidor pede um token.</p>
    <input type="password" placeholder="MINDBLOW_TOKEN" bind:value={v} />
    <button type="submit">Entrar</button>
  </form>
</div>

<style>
  .lock {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: var(--bg);
  }

  form {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    width: min(320px, 86vw);
  }

  .mark {
    width: 18px;
    height: 18px;
    border-radius: 5px;
    background: var(--ink);
    box-shadow: 4px 4px 0 -1px var(--line-strong);
  }

  p {
    margin: 0;
    font: 12px var(--mono);
    color: var(--muted);
  }

  input {
    width: 100%;
    height: 40px;
    padding: 0 12px;
    border-radius: var(--r-md);
    background: var(--paper);
    box-shadow: inset 0 0 0 1px var(--line-strong);
    font: 13px var(--mono);
  }

  button {
    width: 100%;
    height: 40px;
    border: 0;
    border-radius: var(--r-md);
    background: var(--ink);
    color: var(--on-ink);
    font-weight: 500;
    cursor: pointer;
  }
</style>
