<script lang="ts">
  import Icon from '../ui/Icon.svelte'
  import { shell } from '.'
</script>

{#if shell.can.frame}
  <div class="frame">
    <button title="Minimizar" aria-label="Minimizar" onclick={() => shell.minimize()}><Icon name="min" size={14} /></button>
    <button title="Maximizar" aria-label="Maximizar" onclick={() => shell.maximize()}><Icon name="max" size={13} /></button>
    <button class="x" title="Fechar" aria-label="Fechar" onclick={() => shell.close()}><Icon name="x" size={14} /></button>
  </div>
{/if}

<style>
  .frame {
    flex: none;
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: 8px;
  }

  button {
    display: grid;
    place-items: center;
    width: 30px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: none;
    color: var(--muted);
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }

  button:hover {
    background: var(--hover);
    color: var(--fg);
  }

  .x:hover {
    background: var(--tone-1);
    color: #fff;
  }
</style>
