export const W = 212
export const G = 20

export interface Box {
  x: number
  y: number
  h: number
}

export interface Size {
  n: number
  w: number
  g: number
}

export function size(width: number): Size {
  if (width >= 2 * W + G) return { n: Math.floor((width + G) / (W + G)), w: W, g: G }
  const g = 12
  return { n: 2, w: Math.max(120, Math.floor((width - g) / 2)), g }
}

export function layout(hs: number[], s: Size) {
  const col = Array<number>(s.n).fill(0)
  const boxes: Box[] = hs.map((h) => {
    const i = col.indexOf(Math.min(...col))
    const b = { x: i * (s.w + s.g), y: col[i], h }
    col[i] += h + s.g
    return b
  })
  return { boxes, height: Math.max(0, Math.max(...col) - s.g), width: s.n * (s.w + s.g) - s.g }
}

export const HEAD = 34
export const GAP = 32

export function stack(groups: number[][], s: Size, labels: boolean) {
  let top = 0
  const heads: number[] = []
  const boxes = groups.map((hs) => {
    heads.push(top)
    if (!hs.length) return []
    if (labels) top += HEAD
    const m = layout(hs, s)
    const y = top
    top += m.height + GAP
    return m.boxes.map((b) => ({ ...b, y: b.y + y }))
  })
  return { boxes, heads, height: Math.max(0, top - GAP), width: s.n * (s.w + s.g) - s.g }
}

export function near(boxes: Box[], x: number, y: number, w: number): number {
  let best = -1
  let min = w * 0.6
  boxes.forEach((b, i) => {
    const d = Math.hypot(b.x + w / 2 - x, b.y + b.h / 2 - y)
    if (d < min) {
      min = d
      best = i
    }
  })
  return best
}

export function shift<T>(list: T[], from: number, to: number): T[] {
  const v = [...list]
  const [x] = v.splice(from, 1)
  v.splice(to, 0, x)
  return v
}
