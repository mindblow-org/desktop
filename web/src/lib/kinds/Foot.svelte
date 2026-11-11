<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { Name } from '../ui/Icon.svelte'
  import Icon from '../ui/Icon.svelte'

  let { icon, children }: { icon: Name; children: Snippet } = $props()
</script>

<footer>
  <Icon name={icon} size={12} />
  {@render children()}
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 10px;
    font: 10.5px var(--mono);
    color: var(--faint);
    white-space: nowrap;
  }
</style>
