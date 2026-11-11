<script lang="ts" module>
  const paths = {
    grid: ['M4 4h6.5v6.5H4z', 'M13.5 4H20v6.5h-6.5z', 'M4 13.5h6.5V20H4z', 'M13.5 13.5H20V20h-6.5z'],
    plus: ['M12 5v14M5 12h14'],
    x: ['M7 7l10 10M17 7L7 17'],
    detach: ['M14 4h6v6M20 4l-8 8', 'M19 14v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h4'],
    trash: ['M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3'],
    pin: ['M12 16v5M8 3h8l-1 6 3 3v2H6v-2l3-3z'],
    search: ['M11 4.5a6.5 6.5 0 1 1 0 13a6.5 6.5 0 0 1 0-13z', 'M20 20l-4.2-4.2'],
    check: ['M5 12.5l4.5 4.5L19 7.5'],
    back: ['M15 5l-7 7 7 7'],
    next: ['M9 5l7 7-7 7'],
    image: ['M6 5h12a2.5 2.5 0 0 1 2.5 2.5v9A2.5 2.5 0 0 1 18 19H6a2.5 2.5 0 0 1-2.5-2.5v-9A2.5 2.5 0 0 1 6 5z', 'M9 8.4a1.6 1.6 0 1 1 0 3.2a1.6 1.6 0 0 1 0-3.2z', 'M20 16l-4.5-4.5L7 19'],
    home: ['M9 14l-5-5 5-5', 'M4 9h11a5 5 0 0 1 0 10h-3'],
    chev: ['M6 9l6 6 6-6'],
    sticky: ['M5 5h14v9l-5 5H5z', 'M14 19v-5h5'],
    text: ['M5 6h14M5 10h14M5 14h14M5 18h8'],
    todo: ['M8 4h8a4 4 0 0 1 4 4v8a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4V8a4 4 0 0 1 4-4z', 'M8.5 12.2l2.4 2.4 4.6-4.8'],
  }

  export type Name = keyof typeof paths
</script>

<script lang="ts">
  let { name, size = 16, stroke = 1.7 }: { name: Name; size?: number; stroke?: number } = $props()
</script>

<svg width={size} height={size} viewBox="0 0 24 24" stroke-width={stroke} aria-hidden="true">
  {#each paths[name] as d}
    <path {d} />
  {/each}
</svg>

<style>
  svg {
    flex: none;
    stroke: currentColor;
    fill: none;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
