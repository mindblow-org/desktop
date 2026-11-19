<script lang="ts">
  import Icon from '../ui/Icon.svelte'
  import { edge } from './edge'
  import { menu, type Item } from './store.svelte'

  function place(el: HTMLElement) {
    const at = menu.at
    if (!at) return
    const p = edge(at.x, at.y, el.offsetWidth, el.offsetHeight, innerWidth, innerHeight)
    el.style.left = `${p.x}px`
    el.style.top = `${p.y}px`
    el.querySelector('button')?.focus({ preventScroll: true })
    const hide = () => menu.hide()
    const away = (e: PointerEvent) => {
      if (!el.contains(e.target as Node)) hide()
    }
    const key = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.stopPropagation()
      hide()
    }
    addEventListener('pointerdown', away, true)
    addEventListener('keydown', key, true)
    addEventListener('scroll', hide, true)
    addEventListener('resize', hide)
    addEventListener('blur', hide)
    return () => {
      removeEventListener('pointerdown', away, true)
      removeEventListener('keydown', key, true)
      removeEventListener('scroll', hide, true)
      removeEventListener('resize', hide)
      removeEventListener('blur', hide)
    }
  }

  function run(it: Item) {
    menu.hide()
    it.run()
  }
</script>

{#if menu.at}
  <div class="menu" role="menu" {@attach place}>
    {#each menu.at.items as it (it.label)}
      <button role="menuitem" onclick={() => run(it)}><Icon name={it.icon} size={14} />{it.label}</button>
    {/each}
  </div>
{/if}

<style>
  .menu {
    position: fixed;
    z-index: 60;
    min-width: 150px;
    padding: 4px;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    box-shadow: var(--shadow-pop);
    transform-origin: 0 0;
    animation: in 120ms var(--ease);
  }

  @keyframes in {
    from {
      opacity: 0;
      scale: 0.97;
    }
  }

  button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--r-sm);
    background: none;
    color: var(--soft);
    font-size: 13px;
    cursor: pointer;
  }

  button:hover,
  button:focus-visible {
    outline: none;
    background: var(--hover);
    color: var(--fg);
  }
</style>
