// Vetoriza a tinta de uma folha fotografada: o que foi escrito ou desenhado vira traço do board,
// e contorno fechado com cara de caixa ou de círculo vira retângulo ou elipse.
// Caminho: limiar adaptativo, tira pauta e sujeira, afina a tinta até 1 pixel e rastreia as linhas.
import { type El, round } from '../kinds/board/scene'
import { uid } from '../local'

// Foto em RGBA, já reduzida para SIDE no lado maior.
export interface Sheet {
  data: Uint8ClampedArray
  w: number
  h: number
}

// Um traço da folha, em pixels da foto: x, y, x, y…
export interface Line {
  pts: number[]
  c: number
  // A ponta encosta em outro traço: galho curto assim é rebarba do afinamento.
  tied: boolean
}

export interface Page {
  lines: Line[]
  w: number
  h: number
  // Espessura média da tinta, em pixels.
  thick: number
}

export const SIDE = 1400
const WINDOW = 1 / 20 // janela do limiar, em fração do lado maior
const MARGIN = 18 // quanto mais escuro que a vizinhança o pixel precisa ser
const RATIO = 0.15
const EDGE = 0.012 // beira da foto ignorada: sombra e borda do papel
const BAND = 0.03 // mancha que mora quase toda nessa faixa da beira é mesa ou sombra
const RULE = 0.55 // linha fina e clara que atravessa mais que isso da folha é pauta ou margem
const FAINT = 100 // contraste com o papel abaixo disso é tinta clara: pauta ou lápis; caneta é bem mais escura
const SPECK = 10 // mancha menor que isso (em pixels, com a foto em SIDE) é sujeira
const SHAPE = 0.07 // contorno fechado com o lado maior acima disso da folha pode virar forma

export function page(sheet: Sheet): Page {
  const s = crop(sheet)
  const { w, h } = s
  const { m, dark, paper } = ink(s)
  fill(m, w, h)
  rules(m, dark, w, h)
  clean(m, dark, w, h)
  let area = 0
  for (const v of m) area += v
  thin(m, w)
  let len = 0
  for (const v of m) len += v
  const thick = len ? area / len : 0
  prune(m, w, Math.min(6, Math.max(2, Math.round(thick * 0.5))))
  return { lines: trace(m, s, paper, Math.max(2, Math.round(thick * 0.6))), w, h, thick }
}

// Foto crua (sem o scanner do app) pega mesa, teclado e a outra página. A folha é a maior área
// clara e sem cor: recorta nela. Se ela já ocupa quase tudo, ou se não achou folha, fica como está.
export function crop(s: Sheet): Sheet {
  const { w, h, data } = s
  const C = 4
  const cw = Math.floor(w / C)
  const ch = Math.floor(h / C)
  if (cw < 8 || ch < 8) return s
  const lum = new Float32Array(cw * ch)
  const sat = new Float32Array(cw * ch)
  for (let y = 0; y < ch; y++)
    for (let x = 0; x < cw; x++) {
      let r = 0
      let g = 0
      let b = 0
      for (let j = 0; j < C; j++)
        for (let i = 0; i < C; i++) {
          const k = ((y * C + j) * w + x * C + i) * 4
          r += data[k]
          g += data[k + 1]
          b += data[k + 2]
        }
      const hi = Math.max(r, g, b)
      lum[y * cw + x] = (r * 0.3 + g * 0.59 + b * 0.11) / (C * C)
      sat[y * cw + x] = hi ? (hi - Math.min(r, g, b)) / hi : 0
    }
  const top = Float32Array.from(lum).sort()[Math.floor(lum.length * 0.95)]
  const ok = (i: number) => lum[i] > top * 0.5 && sat[i] < 0.3
  const seen = new Uint8Array(cw * ch)
  let best: number[] = []
  let size = 0
  for (let s0 = 0; s0 < seen.length; s0++) {
    if (seen[s0] || !ok(s0)) continue
    const stack = [s0]
    seen[s0] = 1
    const box = [cw, ch, 0, 0]
    let n = 0
    while (stack.length) {
      const i = stack.pop()!
      n++
      const x = i % cw
      const y = (i / cw) | 0
      box[0] = Math.min(box[0], x)
      box[1] = Math.min(box[1], y)
      box[2] = Math.max(box[2], x)
      box[3] = Math.max(box[3], y)
      for (const j of [x > 0 ? i - 1 : -1, x < cw - 1 ? i + 1 : -1, y > 0 ? i - cw : -1, y < ch - 1 ? i + cw : -1])
        if (j >= 0 && !seen[j] && ok(j)) {
          seen[j] = 1
          stack.push(j)
        }
    }
    if (n > size) {
      size = n
      best = box
    }
  }
  const [bx0, by0, bx1, by1] = best
  const area = (bx1 - bx0 + 1) * (by1 - by0 + 1)
  if (!size || size < cw * ch * 0.15 || area > cw * ch * 0.85) return s
  const pad = Math.round(Math.max(bx1 - bx0, by1 - by0) * C * 0.01)
  const x0 = Math.min(w - 1, bx0 * C + pad)
  const y0 = Math.min(h - 1, by0 * C + pad)
  const nw = Math.max(1, Math.min(w, (bx1 + 1) * C - pad) - x0)
  const nh = Math.max(1, Math.min(h, (by1 + 1) * C - pad) - y0)
  const out = new Uint8ClampedArray(nw * nh * 4)
  for (let y = 0; y < nh; y++) out.set(data.subarray(((y0 + y) * w + x0) * 4, ((y0 + y) * w + x0 + nw) * 4), y * nw * 4)
  return { data: out, w: nw, h: nh }
}

// Tinta é o pixel bem mais escuro que a média em volta: aguenta luz desigual e sombra suave.
// dark guarda o contraste de cada pixel de tinta com o papel em volta.
function ink(s: Sheet): { m: Uint8Array; dark: Uint8Array; paper: number[] } {
  const { w, h, data } = s
  const g = new Uint8Array(w * h)
  for (let i = 0, j = 0; i < g.length; i++, j += 4) g[i] = (data[j] * 77 + data[j + 1] * 150 + data[j + 2] * 29) >> 8
  const W = w + 1
  const sum = new Float64Array(W * (h + 1))
  for (let y = 0; y < h; y++) {
    let row = 0
    for (let x = 0; x < w; x++) {
      row += g[y * w + x]
      sum[(y + 1) * W + x + 1] = sum[y * W + x + 1] + row
    }
  }
  const r = Math.max(4, Math.round((Math.max(w, h) * WINDOW) / 2))
  const e = Math.max(1, Math.round(Math.max(w, h) * EDGE))
  const m = new Uint8Array(w * h)
  const dark = new Uint8Array(w * h)
  const paper = [0, 0, 0, 0]
  for (let y = e; y < h - e; y++) {
    const y0 = Math.max(0, y - r)
    const y1 = Math.min(h, y + r + 1)
    for (let x = e; x < w - e; x++) {
      const x0 = Math.max(0, x - r)
      const x1 = Math.min(w, x + r + 1)
      const mean = (sum[y1 * W + x1] - sum[y0 * W + x1] - sum[y1 * W + x0] + sum[y0 * W + x0]) / ((x1 - x0) * (y1 - y0))
      const i = y * w + x
      const d = mean - g[i]
      if (d > Math.max(MARGIN, mean * RATIO)) {
        m[i] = 1
        dark[i] = Math.min(255, d)
      } else {
        paper[0] += data[i * 4]
        paper[1] += data[i * 4 + 1]
        paper[2] += data[i * 4 + 2]
        paper[3]++
      }
    }
  }
  const n = paper[3] || 1
  return { m, dark, paper: [paper[0] / n || 255, paper[1] / n || 255, paper[2] / n || 255] }
}

// Fecha furinhos no meio do traço, que no afinamento viram laços.
function fill(m: Uint8Array, w: number, h: number) {
  const add: number[] = []
  for (let y = 1; y < h - 1; y++)
    for (let x = 1; x < w - 1; x++) {
      const i = y * w + x
      if (!m[i] && count(m, i, w) >= 6) add.push(i)
    }
  for (const i of add) m[i] = 1
}

// Pauta e linha de margem: claras, finas, quase retas e atravessando boa parte da folha.
// Onde a letra cruza a linha a tinta é mais grossa e mais escura que a linha, e esse pedaço fica.
function rules(m: Uint8Array, dark: Uint8Array, w: number, h: number) {
  const thick = Math.max(3, Math.round(Math.max(w, h) * 0.004))
  const [wide, wthin] = strip(m, w, h, true, thick)
  const [tall, tthin] = strip(m, w, h, false, thick)
  for (let i = 0; i < m.length; i++)
    if (dark[i] < FAINT && ((wide[i] && wthin[i]) || (tall[i] && tthin[i]) || (wide[i] && tall[i]))) m[i] = 0
}

// Quem está num caminho longo no eixo, e quem é fino atravessando o eixo.
function strip(m: Uint8Array, w: number, h: number, wide: boolean, thick: number): [Uint8Array, Uint8Array] {
  const A = wide ? w : h
  const B = wide ? h : w
  const at = wide ? (a: number, b: number) => b * w + a : (a: number, b: number) => a * w + b
  // Perto da letra a pauta some no limiar (a média ali é mais escura): o caminho pula esses buracos.
  const jump = Math.max(2, Math.round(Math.max(w, h) * 0.025))
  const k = m.slice()
  for (let b = 0; b < B; b++)
    for (let a = 0, last = -1; a < A; a++) {
      if (!m[at(a, b)]) continue
      if (last >= 0 && a - last > 1 && a - last <= jump) for (let x = last + 1; x < a; x++) k[at(x, b)] = 1
      last = a
    }
  // Comprimento do caminho de tinta que chega a cada pixel andando no eixo, com um pixel de folga para os lados.
  const run = (dir: 1 | -1) => {
    const L = new Uint16Array(m.length)
    for (let a = dir > 0 ? 0 : A - 1; a >= 0 && a < A; a += dir)
      for (let b = 0; b < B; b++) {
        const i = at(a, b)
        if (!k[i]) continue
        let best = 0
        const p = a - dir
        if (p >= 0 && p < A)
          for (let d = -1; d <= 1; d++) {
            const q = b + d
            if (q >= 0 && q < B) best = Math.max(best, L[at(p, q)])
          }
        L[i] = best + 1
      }
    return L
  }
  const fwd = run(1)
  const bwd = run(-1)
  const across = new Uint16Array(m.length)
  for (let a = 0; a < A; a++)
    for (let b = 0; b < B; ) {
      if (!m[at(a, b)]) {
        b++
        continue
      }
      let e = b
      while (e < B && m[at(a, e)]) e++
      for (let k = b; k < e; k++) across[at(a, k)] = e - b
      b = e
    }
  const need = RULE * A
  const long = new Uint8Array(m.length)
  const slim = new Uint8Array(m.length)
  for (let i = 0; i < m.length; i++) {
    if (!m[i]) continue
    if (fwd[i] + bwd[i] - 1 >= need) long[i] = 1
    if (across[i] <= thick) slim[i] = 1
  }
  return [long, slim]
}

// Some com sujeira, com o que mora na beira (mesa, sombra), com o contorno da própria folha
// e com caco de pauta: risquinho fino que nem no miolo fica escuro como caneta.
function clean(m: Uint8Array, dark: Uint8Array, w: number, h: number) {
  const thick = Math.max(3, Math.round(Math.max(w, h) * 0.004)) + 1
  const side = Math.max(w, h)
  const band = Math.round(side * BAND)
  const min = Math.max(3, Math.round(SPECK * (side / SIDE) ** 2))
  const seen = new Uint8Array(m.length)
  const stack: number[] = []
  const blob: number[] = []
  for (let s = 0; s < m.length; s++) {
    if (!m[s] || seen[s]) continue
    blob.length = 0
    stack.push(s)
    seen[s] = 1
    let out = 0
    let deep = 0
    let x0 = w
    let x1 = 0
    let y0 = h
    let y1 = 0
    while (stack.length) {
      const i = stack.pop()!
      blob.push(i)
      deep = Math.max(deep, dark[i])
      const x = i % w
      const y = (i / w) | 0
      if (x < band || x >= w - band || y < band || y >= h - band) out++
      x0 = Math.min(x0, x)
      x1 = Math.max(x1, x)
      y0 = Math.min(y0, y)
      y1 = Math.max(y1, y)
      for (const o of around(w)) {
        const j = i + o
        if (m[j] && !seen[j]) {
          seen[j] = 1
          stack.push(j)
        }
      }
    }
    const bw = x1 - x0 + 1
    const bh = y1 - y0 + 1
    const frame = bw > w * 0.85 && bh > h * 0.85
    const shard = deep < FAINT * 0.8 && bh <= thick && bw >= bh * 4
    if (blob.length < min || out * 2 > blob.length || frame || shard) for (const i of blob) m[i] = 0
  }
}

// Zhang-Suen: come a tinta pelas bordas até sobrar a linha do meio, com 1 pixel.
function thin(m: Uint8Array, w: number) {
  let px: number[] = []
  for (let i = 0; i < m.length; i++) if (m[i]) px.push(i)
  for (let changed = true; changed; ) {
    changed = false
    for (const step of [0, 1]) {
      const del: number[] = []
      for (const i of px) {
        if (!m[i]) continue
        const p2 = m[i - w]
        const p3 = m[i - w + 1]
        const p4 = m[i + 1]
        const p5 = m[i + w + 1]
        const p6 = m[i + w]
        const p7 = m[i + w - 1]
        const p8 = m[i - 1]
        const p9 = m[i - w - 1]
        const b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9
        if (b < 2 || b > 6) continue
        const a =
          +(!p2 && !!p3) + +(!p3 && !!p4) + +(!p4 && !!p5) + +(!p5 && !!p6) + +(!p6 && !!p7) + +(!p7 && !!p8) + +(!p8 && !!p9) + +(!p9 && !!p2)
        if (a !== 1) continue
        if (step === 0 ? p2 * p4 * p6 || p4 * p6 * p8 : p2 * p4 * p8 || p2 * p6 * p8) continue
        del.push(i)
      }
      for (const i of del) m[i] = 0
      if (del.length) changed = true
    }
    px = px.filter((i) => m[i])
  }
  // O Zhang-Suen deixa degraus e trechos com 2 pixels na diagonal, que viram junções falsas.
  // Sai todo pixel que não é ponta e cujos vizinhos continuam ligados entre si sem ele.
  for (let changed = true; changed; ) {
    changed = false
    for (const i of px) {
      if (m[i] && spare(m, i, w)) {
        m[i] = 0
        changed = true
      }
    }
    px = px.filter((i) => m[i])
  }
}

// Vizinhos na ordem N, NE, L, SE, S, SO, O, NO; dois deles se tocam se são seguidos na volta
// ou se ambos são de lado (N e L, por exemplo, se tocam pela diagonal).
const RING = (w: number) => [-w, -w + 1, 1, w + 1, w, w - 1, -1, -w - 1]
const TOUCH = [0, 1, 2, 3, 4, 5, 6, 7].map((a) => [(a + 1) % 8, (a + 7) % 8, ...(a % 2 ? [] : [(a + 2) % 8, (a + 6) % 8])])

function spare(m: Uint8Array, i: number, w: number): boolean {
  const ring = RING(w)
  let set = 0
  let n = 0
  for (let a = 0; a < 8; a++)
    if (m[i + ring[a]]) {
      set |= 1 << a
      n++
    }
  if (n < 2) return false
  const first = 31 - Math.clz32(set)
  let seen = 1 << first
  const stack = [first]
  while (stack.length) for (const b of TOUCH[stack.pop()!]) if (set & (1 << b) && !(seen & (1 << b))) (seen |= 1 << b), stack.push(b)
  return seen === set
}

// Tira rebarbas: come n pixels de cada ponta, devolve às pontas que sobraram o que foi comido
// e traço inteiro mais curto que isso (pingo do i, acento) volta como era.
function prune(m: Uint8Array, w: number, n: number) {
  const px: number[] = []
  for (let i = 0; i < m.length; i++) if (m[i]) px.push(i)
  const orig = m.slice()
  for (let t = 0; t < n; t++) {
    const ends = px.filter((i) => m[i] && count(m, i, w) <= 1)
    if (!ends.length) break
    for (const i of ends) m[i] = 0
  }
  let front = px.filter((i) => m[i] && count(m, i, w) === 1)
  for (let t = 0; t < n && front.length; t++) {
    const next: number[] = []
    for (const i of front)
      for (const o of around(w)) {
        const j = i + o
        if (orig[j] && !m[j]) {
          m[j] = 1
          next.push(j)
        }
      }
    front = next
  }
  const seen = new Uint8Array(m.length)
  const stack: number[] = []
  const blob: number[] = []
  for (const s of px) {
    if (seen[s]) continue
    blob.length = 0
    stack.push(s)
    seen[s] = 1
    let alive = false
    while (stack.length) {
      const i = stack.pop()!
      blob.push(i)
      if (m[i]) alive = true
      for (const o of around(w)) {
        const j = i + o
        if (orig[j] && !seen[j]) {
          seen[j] = 1
          stack.push(j)
        }
      }
    }
    if (!alive) for (const i of blob) m[i] = 1
  }
}

// Rastreia o esqueleto em linhas: começa pelas pontas, segue sempre a continuação mais reta
// e, ao chegar num traço já visto, liga nele. Laço sem ponta (o "o") começa de qualquer pixel.
function trace(m: Uint8Array, s: Sheet, paper: number[], spur: number): Line[] {
  const { w, data } = s
  const O = around(w)
  const DX = [1, 0, -1, 0, 1, -1, -1, 1]
  const DY = [0, 1, 0, -1, 1, 1, -1, -1]
  const px: number[] = []
  for (let i = 0; i < m.length; i++) if (m[i]) px.push(i)
  const seen = new Uint8Array(m.length)

  const walk = (path: number[]) => {
    for (let cur = path[path.length - 1]; ; ) {
      const back = path[Math.max(0, path.length - 5)]
      const vx = (cur % w) - (back % w)
      const vy = ((cur / w) | 0) - ((back / w) | 0)
      let best = -1
      let score = -Infinity
      for (let k = 0; k < 8; k++) {
        const j = cur + O[k]
        if (!m[j] || seen[j]) continue
        const sc = (DX[k] * vx + DY[k] * vy) / Math.hypot(DX[k], DY[k]) - (k >= 4 ? 0.1 : 0)
        if (sc > score) {
          score = sc
          best = j
        }
      }
      if (best < 0) return
      seen[best] = 1
      path.push(best)
      cur = best
    }
  }

  const tie = (path: number[], skip: number[]): boolean => {
    const end = path[path.length - 1]
    const near = new Set([...path.slice(-3), ...skip])
    for (const o of O) {
      const j = end + o
      if (m[j] && seen[j] && !near.has(j)) {
        path.push(j)
        return true
      }
    }
    return false
  }

  const lines: Line[] = []
  const run = (a: number) => {
    seen[a] = 1
    const fwd = [a]
    walk(fwd)
    const t1 = tie(fwd, [])
    const bwd = [a]
    walk(bwd)
    const t2 = tie(bwd, fwd.slice(0, 3))
    const path = bwd.reverse().concat(fwd.slice(1))
    const tied = t1 || t2
    if (tied && path.length < spur) return
    let r = 0
    let g = 0
    let b = 0
    const pts: number[] = []
    for (const i of path) {
      pts.push(i % w, (i / w) | 0)
      r += data[i * 4]
      g += data[i * 4 + 1]
      b += data[i * 4 + 2]
    }
    const n = path.length
    lines.push({ pts, c: tint(r / n / paper[0], g / n / paper[1], b / n / paper[2]), tied })
  }

  for (const i of px) if (!seen[i] && count(m, i, w) === 1) run(i)
  for (const i of px) if (!seen[i]) run(i)
  return lines
}

// Cor da caneta, já descontada a cor do papel (luz amarela não vira tinta âmbar).
// 0 é a cor do texto; 1 a 4 são vermelho, azul, verde e âmbar, como no board.
export function tint(r: number, g: number, b: number): number {
  const hi = Math.max(r, g, b)
  const lo = Math.min(r, g, b)
  if (hi <= 0 || (hi - lo) / hi < 0.35) return 0
  const d = hi - lo
  let hue = hi === r ? ((g - b) / d) % 6 : hi === g ? (b - r) / d + 2 : (r - g) / d + 4
  hue = (hue * 60 + 360) % 360
  if (hue < 25 || hue >= 300) return 1
  if (hue < 70) return 4
  if (hue < 170) return 3
  if (hue < 270) return 2
  return 0
}

// Média com os vizinhos, pontas fixas: tira o degrau dos pixels.
export function smooth(p: number[]): number[] {
  const n = p.length / 2
  if (n < 3) return p.slice()
  const o = p.slice()
  for (let i = 1; i < n - 1; i++) {
    o[i * 2] = (p[i * 2 - 2] + p[i * 2] * 2 + p[i * 2 + 2]) / 4
    o[i * 2 + 1] = (p[i * 2 - 1] + p[i * 2 + 1] * 2 + p[i * 2 + 3]) / 4
  }
  return o
}

// Douglas-Peucker: fica só com os pontos que mudam o desenho mais que eps.
export function simplify(p: number[], eps: number): number[] {
  const n = p.length / 2
  if (n < 3) return p.slice()
  const keep = new Uint8Array(n)
  keep[0] = keep[n - 1] = 1
  const stack = [0, n - 1]
  while (stack.length) {
    const b = stack.pop()!
    const a = stack.pop()!
    const ax = p[a * 2]
    const ay = p[a * 2 + 1]
    const dx = p[b * 2] - ax
    const dy = p[b * 2 + 1] - ay
    const len = Math.hypot(dx, dy)
    let far = -1
    let max = eps
    for (let i = a + 1; i < b; i++) {
      const x = p[i * 2] - ax
      const y = p[i * 2 + 1] - ay
      const d = len ? Math.abs(x * dy - y * dx) / len : Math.hypot(x, y)
      if (d > max) {
        max = d
        far = i
      }
    }
    if (far < 0) continue
    keep[far] = 1
    stack.push(a, far, far, b)
  }
  const o: number[] = []
  for (let i = 0; i < n; i++) if (keep[i]) o.push(p[i * 2], p[i * 2 + 1])
  return o
}

export interface Fit {
  t: 'rect' | 'ellipse'
  x: number
  y: number
  w: number
  h: number
  r: number
}

// Contorno fechado que parece caixa ou círculo. A linha pode começar com um rabo (a ponta de onde
// o rastreio saiu) antes de dar a volta: ele volta junto, para seguir como traço.
export function shape(p: number[], side: number): { fit: Fit; tail: number[] } | null {
  const n = p.length / 2
  if (n < 16) return null
  const acc = [0]
  for (let i = 1; i < n; i++) acc.push(acc[i - 1] + Math.hypot(p[i * 2] - p[i * 2 - 2], p[i * 2 + 1] - p[i * 2 - 1]))
  const total = acc[n - 1]
  const lx = p[n * 2 - 2]
  const ly = p[n * 2 - 1]
  let j = 0
  let gap = Infinity
  for (let i = 0; i < n && total - acc[i] >= total * 0.6; i++) {
    const d = Math.hypot(p[i * 2] - lx, p[i * 2 + 1] - ly)
    if (d < gap) {
      gap = d
      j = i
    }
  }
  const ring = p.slice(j * 2)
  const k = ring.length / 2
  let bx0 = Infinity
  let bx1 = -Infinity
  let by0 = Infinity
  let by1 = -Infinity
  for (let i = 0; i < k; i++) {
    bx0 = Math.min(bx0, ring[i * 2])
    bx1 = Math.max(bx1, ring[i * 2])
    by0 = Math.min(by0, ring[i * 2 + 1])
    by1 = Math.max(by1, ring[i * 2 + 1])
  }
  const big = Math.max(bx1 - bx0, by1 - by0)
  const small = Math.min(bx1 - bx0, by1 - by0)
  if (big < side * SHAPE || small < side * SHAPE * 0.4) return null
  if (gap > small * 0.15 + 3) return null

  const th = tilt(hull(ring))
  const cos = Math.cos(-th)
  const sin = Math.sin(-th)
  const us: number[] = []
  let x0 = Infinity
  let x1 = -Infinity
  let y0 = Infinity
  let y1 = -Infinity
  for (let i = 0; i < k; i++) {
    const u = ring[i * 2] * cos - ring[i * 2 + 1] * sin
    const v = ring[i * 2] * sin + ring[i * 2 + 1] * cos
    us.push(u, v)
    x0 = Math.min(x0, u)
    x1 = Math.max(x1, u)
    y0 = Math.min(y0, v)
    y1 = Math.max(y1, v)
  }
  const W = x1 - x0
  const H = y1 - y0
  const cx = (x0 + x1) / 2
  const cy = (y0 + y1) / 2
  let er = 0
  let ee = 0
  for (let i = 0; i < k; i++) {
    const u = us[i * 2]
    const v = us[i * 2 + 1]
    er += Math.min(u - x0, x1 - u, v - y0, y1 - v)
    ee += Math.abs(Math.hypot((u - cx) / (W / 2), (v - cy) / (H / 2)) - 1)
  }
  er /= (k * Math.min(W, H)) / 2
  ee /= k
  const reach = Math.min(W, H) * 0.18
  const corners = [
    [x0, y0],
    [x1, y0],
    [x1, y1],
    [x0, y1],
  ].every(([qx, qy]) => {
    for (let i = 0; i < k; i++) if (Math.hypot(us[i * 2] - qx, us[i * 2 + 1] - qy) < reach) return true
    return false
  })
  const t = corners && er < 0.1 ? 'rect' : ee < 0.1 ? 'ellipse' : null
  if (!t) return null
  const X = cx * Math.cos(th) - cy * Math.sin(th)
  const Y = cx * Math.sin(th) + cy * Math.cos(th)
  return { fit: { t, x: X - W / 2, y: Y - H / 2, w: W, h: H, r: Math.abs(th) < 0.035 ? 0 : th }, tail: p.slice(0, j * 2 + 2) }
}

// Fecho convexo (cadeia monótona).
function hull(p: number[]): number[][] {
  const pts: number[][] = []
  for (let i = 0; i < p.length; i += 2) pts.push([p[i], p[i + 1]])
  pts.sort((a, b) => a[0] - b[0] || a[1] - b[1])
  const cross = (o: number[], a: number[], b: number[]) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
  const lo: number[][] = []
  for (const q of pts) {
    while (lo.length >= 2 && cross(lo[lo.length - 2], lo[lo.length - 1], q) <= 0) lo.pop()
    lo.push(q)
  }
  const up: number[][] = []
  for (const q of pts.reverse()) {
    while (up.length >= 2 && cross(up[up.length - 2], up[up.length - 1], q) <= 0) up.pop()
    up.push(q)
  }
  return lo.slice(0, -1).concat(up.slice(0, -1))
}

// Inclinação do menor retângulo que cobre o fecho, entre -45° e 45°.
function tilt(h: number[][]): number {
  let best = Infinity
  let th = 0
  for (let i = 0; i < h.length; i++) {
    const a = h[i]
    const b = h[(i + 1) % h.length]
    const ang = Math.atan2(b[1] - a[1], b[0] - a[0])
    const c = Math.cos(-ang)
    const s = Math.sin(-ang)
    let x0 = Infinity
    let x1 = -Infinity
    let y0 = Infinity
    let y1 = -Infinity
    for (const q of h) {
      const u = q[0] * c - q[1] * s
      const v = q[0] * s + q[1] * c
      x0 = Math.min(x0, u)
      x1 = Math.max(x1, u)
      y0 = Math.min(y0, v)
      y1 = Math.max(y1, v)
    }
    const area = (x1 - x0) * (y1 - y0)
    if (area < best) {
      best = area
      th = ang
    }
  }
  while (th > Math.PI / 4) th -= Math.PI / 2
  while (th <= -Math.PI / 4) th += Math.PI / 2
  return th
}

// Linhas da folha viram elementos do board: escala k, deslocadas por (dx, dy).
export function draw(pg: Page, o: { eps: number; k: number; dx: number; dy: number }): El[] {
  const side = Math.max(pg.w, pg.h)
  const at = (p: number[]) => p.map((v, i) => v * o.k + (i % 2 ? o.dy : o.dx))
  const pen = (p: number[], c: number): El => round({ id: uid(), t: 'pen', c, pts: at(p.length > 4 ? simplify(p, o.eps) : p) })
  const els: El[] = []
  for (const l of pg.lines) {
    const p = smooth(l.pts)
    const f = shape(p, side)
    if (!f) {
      els.push(pen(p, l.c))
      continue
    }
    const { t, x, y, w, h, r } = f.fit
    els.push(round({ id: uid(), t, c: l.c, x: x * o.k + o.dx, y: y * o.k + o.dy, w: w * o.k, h: h * o.k, ...(r ? { r } : {}) }))
    if (f.tail.length / 2 > Math.max(2, pg.thick * 2)) els.push(pen(f.tail, l.c))
  }
  return els
}

function count(m: Uint8Array, i: number, w: number): number {
  return m[i - w - 1] + m[i - w] + m[i - w + 1] + m[i - 1] + m[i + 1] + m[i + w - 1] + m[i + w] + m[i + w + 1]
}

// Vizinhos: os quatro de lado primeiro, depois as diagonais (o rastreio prefere andar reto).
function around(w: number): number[] {
  return [1, w, -1, -w, w + 1, w - 1, -w - 1, -w + 1]
}
