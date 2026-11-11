import type { Note } from '../notes/types'

export interface Part {
  t: string
  on: boolean
}

export interface Hit {
  note: Note
  title: Part[]
  snip: Part[]
  score: number
}

const AROUND = 32
const LONG = 110

export function fold(s: string): string {
  let out = ''
  for (const c of s) {
    const f = c.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase()
    out += f.length === c.length ? f : c
  }
  return out
}

export function terms(q: string): string[] {
  return fold(q).split(/\s+/).filter(Boolean)
}

export function search(list: Note[], q: string, name: (n: Note) => string, max = 30): Hit[] {
  const ts = terms(q)
  if (!ts.length) {
    return [...list]
      .sort((a, b) => b.updated - a.updated)
      .slice(0, max)
      .map((n) => ({ note: n, title: plain(name(n)), snip: plain(cut(rest(n, name(n)), 0)), score: 0 }))
  }
  const hits: Hit[] = []
  for (const n of list) {
    const title = name(n)
    const body = rest(n, title)
    const ft = fold(title)
    const fb = fold(body)
    if (!ts.every((t) => ft.includes(t) || fb.includes(t))) continue
    const score = ts.reduce((s, t) => s + (ft.startsWith(t) ? 3 : ft.includes(t) ? 2 : 0), 0)
    hits.push({ note: n, title: mark(title, ft, ts), snip: snip(body, fb, ts), score })
  }
  return hits.sort((a, b) => b.score - a.score || b.note.updated - a.note.updated).slice(0, max)
}

function rest(n: Note, title: string): string {
  const parts = [n.body, ...n.items.map((i) => i.text)].map((s) => s.replace(/\s+/g, ' ').trim()).filter(Boolean)
  const text = parts.join(' · ')
  return text === title ? '' : text
}

function snip(body: string, fb: string, ts: string[]): Part[] {
  const hit = Math.min(...ts.map((t) => fb.indexOf(t)).filter((i) => i >= 0))
  if (!Number.isFinite(hit)) return plain(cut(body, 0))
  let from = Math.max(0, hit - AROUND)
  if (from > 0) from = body.lastIndexOf(' ', from) + 1
  const end = Math.min(body.length, from + LONG)
  const pre = from > 0 ? '…' : ''
  return [...(pre ? [{ t: pre, on: false }] : []), ...mark(body.slice(from, end), fb.slice(from, end), ts)]
}

function cut(s: string, from: number): string {
  return s.slice(from, from + LONG)
}

function plain(t: string): Part[] {
  return t ? [{ t, on: false }] : []
}

export function mark(s: string, fs: string, ts: string[]): Part[] {
  const on = new Array<boolean>(s.length).fill(false)
  for (const t of ts) {
    for (let i = fs.indexOf(t); i >= 0; i = fs.indexOf(t, i + t.length)) on.fill(true, i, i + t.length)
  }
  const parts: Part[] = []
  for (let i = 0; i < s.length; i++) {
    const last = parts.at(-1)
    if (last && last.on === on[i]) last.t += s[i]
    else parts.push({ t: s[i], on: on[i] })
  }
  return parts
}
