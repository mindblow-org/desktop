<script lang="ts" module>
  export const tones = ['var(--ink)', 'var(--tone-1)', 'var(--tone-2)', 'var(--tone-3)', 'var(--tone-4)']
</script>

<script lang="ts">
  import { src } from '../../api/http'
  import { angle, center, type El, font, head, LINE, lines, path } from './scene'

  let { els, hide = null, thumb = false }: { els: El[]; hide?: string | null; thumb?: boolean } = $props()

  function spin(e: El) {
    const a = angle(e)
    if (!a) return undefined
    const c = center(e)
    return `rotate(${(a * 180) / Math.PI} ${c.x} ${c.y})`
  }
</script>

<g class="shapes" class:thumb>
  {#each els as e (e.id)}
    {#if e.id !== hide}
      {@const c = tones[e.c] ?? tones[0]}
      <g transform={spin(e)}>
        {#if e.t === 'pen'}
          {#if e.pts.length === 2}
            <circle class="dot" cx={e.pts[0]} cy={e.pts[1]} r="1.6" style:fill={c} />
          {:else}
            <path class="pen" d={path(e.pts)} style:stroke={c} />
          {/if}
        {:else if e.t === 'rect'}
          <rect x={e.x} y={e.y} width={e.w} height={e.h} rx="4" style:stroke={c} />
        {:else if e.t === 'ellipse'}
          <ellipse cx={e.x + e.w / 2} cy={e.y + e.h / 2} rx={e.w / 2} ry={e.h / 2} style:stroke={c} />
        {:else if e.t === 'arrow'}
          <path d="M{e.x1} {e.y1}L{e.x2} {e.y2}{head(e)}" style:stroke={c} />
        {:else if e.t === 'image'}
          <image href={src(e.img)} x={e.x} y={e.y} width={e.w} height={e.h} preserveAspectRatio="none" />
        {:else if e.t === 'text'}
          {@const f = font(e)}
          <text x={e.x} y={e.y} font-size={f} style:fill={c}>
            {#each lines(e.s) as l, i (i)}<tspan x={e.x} dy={i ? f * LINE : f * 1.02}>{l || ' '}</tspan>{/each}
          </text>
        {/if}
      </g>
    {/if}
  {/each}
</g>

<style>
  .shapes {
    fill: none;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .pen {
    stroke-width: 2.4;
  }

  .dot {
    stroke: none;
  }

  text {
    stroke: none;
    font-family: var(--sans);
    white-space: pre;
  }

  .thumb :is(path, rect, ellipse) {
    vector-effect: non-scaling-stroke;
    stroke-width: 1.5;
  }
</style>
