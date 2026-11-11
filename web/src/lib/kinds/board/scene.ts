export type Tool = 'select' | 'hand' | 'pen' | 'rect' | 'ellipse' | 'arrow' | 'text'
export type Shape = 'rect' | 'ellipse'

interface Base {
  id: string
  c: number
  r?: number
}

export type Pen = Base & { t: 'pen'; pts: number[] }
export type Frame = Base & { t: Shape; x: number; y: number; w: number; h: number }
export type Pic = Base & { t: 'image'; img: string; x: number; y: number; w: number; h: number }
export type Arrow = Base & { t: 'arrow'; x1: number; y1: number; x2: number; y2: number }
export type Label = Base & { t: 'text'; x: number; y: number; s: string; f?: number }
export type El = Pen | Frame | Pic | Arrow | Label

export interface Pt {
  x: number
  y: number
}

export interface Box {
  x: number
  y: number
  w: number
  h: number
}

export const FONT = 20
export const LINE = 1.25
const GLYPH = 0.56
const MIN = 4

let ctx: CanvasRenderingContext2D | null | undefined
let family = ''
const widths = new Map<string, number>()

function measure(s: string, f: number): number {
  if (ctx === undefined) {
    ctx = typeof document === 'undefined' ? null : document.createElement('canvas').getContext('2d')
    if (ctx) family = getComputedStyle(document.documentElement).getPropertyValue('--sans').trim() || 'sans-serif'
  }
  if (!ctx) return s.length * f * GLYPH
  const k = `${f}|${s}`
  let w = widths.get(k)
  if (w === undefined) {
    if (widths.size > 500) widths.clear()
    ctx.font = `${f}px ${family}`
    w = ctx.measureText(s).width
    widths.set(k, w)
  }
  return w
}

export function read(body: string): El[] {
  try {
    const els = JSON.parse(body)?.els
    return Array.isArray(els) ? els : []
  } catch {
    return []
  }
}

export function write(els: El[]): string {
  return els.length ? JSON.stringify({ v: 1, els }) : ''
}

export function words(els: El[]): string {
  return els
    .filter((e): e is Label => e.t === 'text')
    .map((e) => e.s)
    .join(' · ')
}

export function frame(x1: number, y1: number, x2: number, y2: number): Box {
  return { x: Math.min(x1, x2), y: Math.min(y1, y2), w: Math.abs(x2 - x1), h: Math.abs(y2 - y1) }
}

export function lines(s: string): string[] {
  return s.split('\n')
}

export function font(e: Label): number {
  return e.f ?? FONT
}

export function angle(e: El): number {
  return e.t === 'arrow' ? 0 : (e.r ?? 0)
}

export function rot(p: Pt, c: Pt, a: number): Pt {
  if (!a) return p
  const cos = Math.cos(a)
  const sin = Math.sin(a)
  const dx = p.x - c.x
  const dy = p.y - c.y
  return { x: c.x + dx * cos - dy * sin, y: c.y + dx * sin + dy * cos }
}

export function local(e: El): Box {
  switch (e.t) {
    case 'pen': {
      const xs = e.pts.filter((_, i) => i % 2 === 0)
      const ys = e.pts.filter((_, i) => i % 2 === 1)
      return frame(Math.min(...xs), Math.min(...ys), Math.max(...xs), Math.max(...ys))
    }
    case 'arrow':
      return frame(e.x1, e.y1, e.x2, e.y2)
    case 'text': {
      const f = font(e)
      const ls = lines(e.s)
      return { x: e.x, y: e.y, w: Math.max(f * 0.5, ...ls.map((l) => measure(l, f))), h: ls.length * f * LINE }
    }
    default:
      return { x: e.x, y: e.y, w: e.w, h: e.h }
  }
}

export function center(e: El): Pt {
  const b = local(e)
  return { x: b.x + b.w / 2, y: b.y + b.h / 2 }
}

export function corners(e: El): Pt[] {
  const b = local(e)
  const c = { x: b.x + b.w / 2, y: b.y + b.h / 2 }
  const a = angle(e)
  return [
    { x: b.x, y: b.y },
    { x: b.x + b.w, y: b.y },
    { x: b.x + b.w, y: b.y + b.h },
    { x: b.x, y: b.y + b.h },
  ].map((p) => rot(p, c, a))
}

function hull(ps: Pt[]): Box {
  const xs = ps.map((p) => p.x)
  const ys = ps.map((p) => p.y)
  return frame(Math.min(...xs), Math.min(...ys), Math.max(...xs), Math.max(...ys))
}

export function box(e: El): Box {
  return angle(e) ? hull(corners(e)) : local(e)
}

export function bounds(els: El[]): Box | null {
  if (!els.length) return null
  return hull(els.flatMap((e) => {
    const b = box(e)
    return [{ x: b.x, y: b.y }, { x: b.x + b.w, y: b.y + b.h }]
  }))
}

function seg(px: number, py: number, x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1
  const dy = y2 - y1
  const len = dx * dx + dy * dy
  const t = len ? Math.max(0, Math.min(1, ((px - x1) * dx + (py - y1) * dy) / len)) : 0
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy))
}

function within(b: Box, x: number, y: number, tol: number): boolean {
  return x >= b.x - tol && x <= b.x + b.w + tol && y >= b.y - tol && y <= b.y + b.h + tol
}

export function hit(e: El, px: number, py: number, tol: number): boolean {
  const { x, y } = rot({ x: px, y: py }, center(e), -angle(e))
  switch (e.t) {
    case 'pen': {
      const p = e.pts
      if (p.length === 2) return Math.hypot(x - p[0], y - p[1]) <= tol
      for (let i = 2; i < p.length; i += 2) if (seg(x, y, p[i - 2], p[i - 1], p[i], p[i + 1]) <= tol) return true
      return false
    }
    case 'arrow':
      return seg(x, y, e.x1, e.y1, e.x2, e.y2) <= tol
    case 'rect': {
      const { x: l, y: t, w, h } = e
      const r = l + w
      const b = t + h
      return [seg(x, y, l, t, r, t), seg(x, y, r, t, r, b), seg(x, y, l, b, r, b), seg(x, y, l, t, l, b)].some((d) => d <= tol)
    }
    case 'ellipse': {
      const rx = e.w / 2
      const ry = e.h / 2
      if (!rx || !ry) return seg(x, y, e.x, e.y, e.x + e.w, e.y + e.h) <= tol
      const d = Math.hypot((x - e.x - rx) / rx, (y - e.y - ry) / ry)
      return Math.abs(d - 1) * Math.min(rx, ry) <= tol
    }
    default:
      return within(local(e), x, y, tol)
  }
}

export function at(els: El[], x: number, y: number, tol: number): El | undefined {
  for (let i = els.length - 1; i >= 0; i--) if (hit(els[i], x, y, tol)) return els[i]
}

export function inside(els: El[], area: Box): El[] {
  return els.filter((e) => {
    const b = box(e)
    return b.x >= area.x && b.y >= area.y && b.x + b.w <= area.x + area.w && b.y + b.h <= area.y + area.h
  })
}

export function grabs(e: El, x: number, y: number, tol: number): boolean {
  if (e.t === 'arrow') return hit(e, x, y, tol)
  const p = rot({ x, y }, center(e), -angle(e))
  return within(local(e), p.x, p.y, tol)
}

export function thin(pts: number[], min = 2): number[] {
  if (pts.length < 2) return []
  const out = [Math.round(pts[0]), Math.round(pts[1])]
  for (let i = 2; i < pts.length; i += 2) {
    const x = Math.round(pts[i])
    const y = Math.round(pts[i + 1])
    if (Math.hypot(x - out[out.length - 2], y - out[out.length - 1]) >= min) out.push(x, y)
  }
  const lx = Math.round(pts[pts.length - 2])
  const ly = Math.round(pts[pts.length - 1])
  if (out.length > 2 && (out[out.length - 2] !== lx || out[out.length - 1] !== ly)) out.push(lx, ly)
  return out
}

export function move(e: El, dx: number, dy: number): El {
  switch (e.t) {
    case 'pen':
      return { ...e, pts: e.pts.map((v, i) => v + (i % 2 ? dy : dx)) }
    case 'arrow':
      return { ...e, x1: e.x1 + dx, y1: e.y1 + dy, x2: e.x2 + dx, y2: e.y2 + dy }
    default:
      return { ...e, x: e.x + dx, y: e.y + dy }
  }
}

function norm(a: number): number {
  const t = Math.PI * 2
  const n = ((a % t) + t) % t
  return n > Math.PI ? n - t : n
}

export function turn(e: El, a: number, pivot: Pt): El {
  if (e.t === 'arrow') {
    const p = rot({ x: e.x1, y: e.y1 }, pivot, a)
    const q = rot({ x: e.x2, y: e.y2 }, pivot, a)
    return { ...e, x1: p.x, y1: p.y, x2: q.x, y2: q.y }
  }
  const c = center(e)
  const n = rot(c, pivot, a)
  return { ...move(e, n.x - c.x, n.y - c.y), r: norm(angle(e) + a) } as El
}

function reshape(e: El, c: Pt, kx: number, ky: number): El {
  const old = center(e)
  switch (e.t) {
    case 'pen':
      return { ...e, pts: e.pts.map((v, i) => (i % 2 ? c.y + (v - old.y) * ky : c.x + (v - old.x) * kx)) }
    case 'arrow':
      return { ...e, x1: c.x + (e.x1 - old.x) * kx, y1: c.y + (e.y1 - old.y) * ky, x2: c.x + (e.x2 - old.x) * kx, y2: c.y + (e.y2 - old.y) * ky }
    case 'text': {
      const b = local(e)
      return { ...e, f: font(e) * ky, x: c.x - (b.w * ky) / 2, y: c.y - (b.h * ky) / 2 }
    }
    default: {
      const w = e.w * kx
      const h = e.h * ky
      return { ...e, x: c.x - w / 2, y: c.y - h / 2, w, h }
    }
  }
}

export function grow(e: El, s: number, pivot: Pt): El {
  const c = center(e)
  return reshape(e, { x: pivot.x + (c.x - pivot.x) * s, y: pivot.y + (c.y - pivot.y) * s }, s, s)
}

export function locked(e: El): boolean {
  return e.t === 'text' || e.t === 'image'
}

export function resize(e: El, corner: number, p: Pt, keep: boolean): El {
  const b = local(e)
  const a = angle(e)
  const c = { x: b.x + b.w / 2, y: b.y + b.h / 2 }
  const sx = corner === 1 || corner === 2 ? 1 : -1
  const sy = corner >= 2 ? 1 : -1
  const fixed = rot({ x: c.x - (sx * b.w) / 2, y: c.y - (sy * b.h) / 2 }, c, a)
  const d = rot({ x: p.x - fixed.x, y: p.y - fixed.y }, { x: 0, y: 0 }, -a)
  let w = Math.max(MIN, sx * d.x)
  let h = Math.max(MIN, sy * d.y)
  if (keep || locked(e)) {
    const s = Math.max(w / Math.max(b.w, 1), h / Math.max(b.h, 1))
    w = b.w * s
    h = b.h * s
  }
  const off = rot({ x: (sx * w) / 2, y: (sy * h) / 2 }, { x: 0, y: 0 }, a)
  const kx = b.w >= 1 ? w / b.w : 1
  const ky = b.h >= 1 ? h / b.h : 1
  return reshape(e, { x: fixed.x + off.x, y: fixed.y + off.y }, kx, ky)
}

export function tip(e: Arrow, end: 1 | 2, p: Pt): Arrow {
  return end === 1 ? { ...e, x1: p.x, y1: p.y } : { ...e, x2: p.x, y2: p.y }
}

export function round(e: El): El {
  const r = (v: number) => Math.round(v)
  const a = e.t === 'arrow' || !e.r ? {} : { r: Math.round(e.r * 1000) / 1000 }
  switch (e.t) {
    case 'pen':
      return { ...e, ...a, pts: e.pts.map(r) }
    case 'arrow':
      return { ...e, x1: r(e.x1), y1: r(e.y1), x2: r(e.x2), y2: r(e.y2) }
    case 'text':
      return { ...e, ...a, x: r(e.x), y: r(e.y), ...(e.f ? { f: Math.round(e.f * 10) / 10 } : {}) }
    default:
      return { ...e, ...a, x: r(e.x), y: r(e.y), w: r(e.w), h: r(e.h) }
  }
}

export function tiny(e: El): boolean {
  switch (e.t) {
    case 'pen':
      return e.pts.length < 2
    case 'arrow':
      return Math.hypot(e.x2 - e.x1, e.y2 - e.y1) < 4
    case 'text':
      return !e.s.trim()
    default:
      return e.w < 3 && e.h < 3
  }
}

export function clone(els: El[], dx: number, dy: number, id: () => string): El[] {
  return els.map((e) => ({ ...move(e, dx, dy), id: id() }))
}

export function path(pts: number[]): string {
  if (pts.length < 4) return ''
  let d = `M${pts[0]} ${pts[1]}`
  for (let i = 2; i < pts.length - 2; i += 2) {
    const mx = (pts[i] + pts[i + 2]) / 2
    const my = (pts[i + 1] + pts[i + 3]) / 2
    d += `Q${pts[i]} ${pts[i + 1]} ${mx} ${my}`
  }
  return `${d}L${pts[pts.length - 2]} ${pts[pts.length - 1]}`
}

export function head(e: Arrow, size = 14): string {
  const a = Math.atan2(e.y2 - e.y1, e.x2 - e.x1)
  const s = Math.min(size, Math.hypot(e.x2 - e.x1, e.y2 - e.y1) / 2.5)
  const p = (k: number) => `${e.x2 - s * Math.cos(a + k)} ${e.y2 - s * Math.sin(a + k)}`
  return `M${p(0.45)}L${e.x2} ${e.y2}L${p(-0.45)}`
}
