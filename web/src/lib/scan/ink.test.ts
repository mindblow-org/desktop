import { describe, expect, it } from 'vitest'
import type { El, Frame, Pen } from '../kinds/board/scene'
import { crop, draw, page, type Sheet, simplify, tint } from './ink'

type RGB = [number, number, number]
const PAPER: RGB = [246, 244, 238]
const BLACK: RGB = [35, 35, 40]

function blank(w: number, h: number, bg: RGB = PAPER): Sheet {
  const data = new Uint8ClampedArray(w * h * 4)
  for (let i = 0; i < w * h; i++) data.set([...bg, 255], i * 4)
  return { data, w, h }
}

function dot(s: Sheet, x: number, y: number, r: number, c: RGB) {
  for (let j = Math.floor(y - r); j <= Math.ceil(y + r); j++)
    for (let i = Math.floor(x - r); i <= Math.ceil(x + r); i++)
      if (i >= 0 && j >= 0 && i < s.w && j < s.h && Math.hypot(i - x, j - y) <= r) s.data.set(c, (j * s.w + i) * 4)
}

function line(s: Sheet, x0: number, y0: number, x1: number, y1: number, c: RGB = BLACK, r = 2) {
  const n = Math.ceil(Math.hypot(x1 - x0, y1 - y0))
  for (let k = 0; k <= n; k++) dot(s, x0 + ((x1 - x0) * k) / n, y0 + ((y1 - y0) * k) / n, r, c)
}

function box(s: Sheet, x: number, y: number, w: number, h: number, c: RGB = BLACK) {
  line(s, x, y, x + w, y, c)
  line(s, x + w, y, x + w, y + h, c)
  line(s, x + w, y + h, x, y + h, c)
  line(s, x, y + h, x, y, c)
}

function ring(s: Sheet, cx: number, cy: number, rx: number, ry: number, c: RGB = BLACK) {
  for (let a = 0; a < Math.PI * 2; a += 0.004) dot(s, cx + rx * Math.cos(a), cy + ry * Math.sin(a), 2, c)
}

function els(s: Sheet): El[] {
  return draw(page(s), { eps: 0.9, k: 1, dx: 0, dy: 0 })
}

const pens = (e: El[]) => e.filter((x): x is Pen => x.t === 'pen')
const frames = (e: El[]) => e.filter((x): x is Frame => x.t === 'rect' || x.t === 'ellipse')

describe('escanear folha', () => {
  it('folha em branco não vira nada', () => {
    expect(els(blank(400, 300))).toEqual([])
  })

  it('um risco vira um traço só, de ponta a ponta', () => {
    const s = blank(400, 300)
    line(s, 80, 100, 320, 200)
    const p = pens(els(s))
    expect(p).toHaveLength(1)
    const [x0, y0] = p[0].pts
    const [x1, y1] = p[0].pts.slice(-2)
    const ends = [
      [x0, y0],
      [x1, y1],
    ].sort((a, b) => a[0] - b[0])
    expect(Math.hypot(ends[0][0] - 80, ends[0][1] - 100)).toBeLessThan(6)
    expect(Math.hypot(ends[1][0] - 320, ends[1][1] - 200)).toBeLessThan(6)
  })

  it('traço reto fica com poucos pontos', () => {
    const s = blank(400, 300)
    line(s, 60, 150, 340, 150)
    expect(pens(els(s))[0].pts.length).toBeLessThanOrEqual(8)
  })

  it('a pauta do caderno some e a letra que cruza ela fica', () => {
    const s = blank(500, 400)
    for (let y = 40; y < 400; y += 30) line(s, 0, y, 499, y, [175, 195, 225], 0.8)
    line(s, 40, 0, 40, 399, [225, 140, 150], 0.8)
    line(s, 200, 90, 260, 200)
    const e = els(s)
    expect(e).toHaveLength(1)
    const p = pens(e)[0].pts
    const ys = p.filter((_, i) => i % 2)
    expect(Math.min(...ys)).toBeLessThan(100)
    expect(Math.max(...ys)).toBeGreaterThan(190)
  })

  it('sombra na beira da foto não vira traço', () => {
    const s = blank(500, 400)
    for (let y = 0; y < 400; y++) for (let x = 0; x < 18; x++) s.data.set([60, 60, 60], (y * 500 + x) * 4)
    line(s, 150, 200, 350, 220)
    expect(els(s)).toHaveLength(1)
  })

  it('sujeira solta some, pingo do i fica', () => {
    const s = blank(400, 300)
    s.data.set(BLACK, (100 * 400 + 100) * 4)
    s.data.set(BLACK, (220 * 400 + 300) * 4)
    line(s, 200, 160, 200, 230)
    dot(s, 200, 140, 3, BLACK)
    expect(pens(els(s))).toHaveLength(2)
  })

  it('caixa desenhada vira retângulo', () => {
    const s = blank(600, 500)
    box(s, 120, 100, 300, 180)
    const e = els(s)
    expect(e).toHaveLength(1)
    const f = frames(e)[0]
    expect(f.t).toBe('rect')
    expect(Math.abs(f.x - 120)).toBeLessThan(6)
    expect(Math.abs(f.y - 100)).toBeLessThan(6)
    expect(Math.abs(f.w - 300)).toBeLessThan(10)
    expect(Math.abs(f.h - 180)).toBeLessThan(10)
    expect(f.r ?? 0).toBe(0)
  })

  it('caixa torta vira retângulo girado', () => {
    const s = blank(600, 500)
    const a = 0.25
    const c = [300, 250]
    const q = [
      [-150, -90],
      [150, -90],
      [150, 90],
      [-150, 90],
    ].map(([x, y]) => [c[0] + x * Math.cos(a) - y * Math.sin(a), c[1] + x * Math.sin(a) + y * Math.cos(a)])
    for (let i = 0; i < 4; i++) line(s, q[i][0], q[i][1], q[(i + 1) % 4][0], q[(i + 1) % 4][1])
    const f = frames(els(s))[0]
    expect(f?.t).toBe('rect')
    expect(Math.abs((f.r ?? 0) - a)).toBeLessThan(0.05)
    expect(Math.abs(f.x + f.w / 2 - 300)).toBeLessThan(6)
  })

  it('círculo desenhado vira elipse', () => {
    const s = blank(600, 500)
    ring(s, 300, 250, 140, 90)
    const f = frames(els(s))
    expect(f).toHaveLength(1)
    expect(f[0].t).toBe('ellipse')
    expect(Math.abs(f[0].w - 280)).toBeLessThan(10)
    expect(Math.abs(f[0].h - 180)).toBeLessThan(10)
  })

  it('letra "o" pequena continua traço', () => {
    const s = blank(1000, 700)
    ring(s, 400, 300, 14, 12)
    const e = els(s)
    expect(frames(e)).toHaveLength(0)
    expect(pens(e).length).toBeGreaterThan(0)
  })

  it('triângulo não vira forma', () => {
    const s = blank(600, 500)
    line(s, 300, 80, 460, 380)
    line(s, 460, 380, 140, 380)
    line(s, 140, 380, 300, 80)
    expect(frames(els(s))).toHaveLength(0)
  })

  it('caneta azul e vermelha mantêm a cor; preta e lápis ficam na cor do texto', () => {
    const s = blank(600, 400)
    line(s, 60, 80, 540, 90, [30, 45, 120])
    line(s, 60, 180, 540, 190, [190, 30, 35])
    line(s, 60, 280, 540, 290, BLACK)
    line(s, 60, 350, 540, 360, [120, 120, 118])
    const c = pens(els(s))
      .sort((a, b) => a.pts[1] - b.pts[1])
      .map((p) => p.c)
    expect(c).toEqual([2, 1, 0, 0])
  })

  it('luz amarela não pinta a tinta preta de âmbar', () => {
    expect(tint(60 / 235, 50 / 215, 35 / 170)).toBe(0)
  })

  it('cena sai com números inteiros', () => {
    const s = blank(400, 300)
    line(s, 50, 50, 350, 250)
    const p = draw(page(s), { eps: 0.9, k: 0.37, dx: 3, dy: 1000 })
    for (const e of pens(p)) for (const v of e.pts) expect(Number.isInteger(v)).toBe(true)
    expect(Math.min(...pens(p)[0].pts.filter((_, i) => i % 2))).toBeGreaterThan(1000)
  })

  it('foto crua recorta na folha: mesa e teclado ficam de fora', () => {
    const s = blank(400, 600, [200, 168, 120])
    for (let y = 0; y < 600; y++)
      for (let x = 0; x < 400; x++) if (y > 520) s.data.set([30, 30, 34], (y * 400 + x) * 4)
        else if (x >= 60 && x < 340 && y >= 80 && y < 480) s.data.set(PAPER, (y * 400 + x) * 4)
    const c = crop(s)
    expect(c.w).toBeGreaterThan(250)
    expect(c.w).toBeLessThan(285)
    expect(c.h).toBeGreaterThan(370)
    expect(c.h).toBeLessThan(405)
  })

  it('folha que já ocupa a foto toda não é recortada', () => {
    const s = blank(400, 300)
    line(s, 50, 50, 350, 250)
    expect(crop(s)).toBe(s)
  })

  it('simplificar mantém as pontas e o canto', () => {
    expect(simplify([0, 0, 1, 0, 2, 0, 3, 0, 3, 1, 3, 2, 3, 3], 0.5)).toEqual([0, 0, 3, 0, 3, 3])
  })
})
