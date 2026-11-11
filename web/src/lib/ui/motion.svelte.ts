import { tick } from 'svelte'
import { cubicOut } from 'svelte/easing'

export const fly = $state<{ id: string | null }>({ id: null })

export function go(f: () => void, id: string | null = null) {
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches
  if (!document.startViewTransition || still) return f()
  fly.id = id
  const t = document.startViewTransition(async () => {
    f()
    await tick()
  })
  t.finished.finally(() => (fly.id = null))
}

export function grow(node: HTMLElement, { duration = 180 } = {}) {
  const w = node.offsetWidth
  return {
    duration,
    easing: cubicOut,
    css: (t: number) => `max-width:${t * w}px;opacity:${t};translate:${(t - 1) * 8}px 0;`,
  }
}

export function pop(_: Element, { duration = 200 } = {}) {
  return {
    duration,
    easing: cubicOut,
    css: (t: number) => `opacity:${t};scale:${0.94 + 0.06 * t};`,
  }
}
