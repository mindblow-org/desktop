<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { src } from '../../api/http'
  import { uid } from '../../local'
  import { store } from '../../notes/store.svelte'
  import type { Note } from '../../notes/types'
  import Icon, { type Name } from '../../ui/Icon.svelte'
  import Meta from '../Meta.svelte'
  import {
    angle,
    at,
    type Box,
    bounds,
    center,
    clone,
    corners,
    type El,
    FONT,
    font,
    frame,
    grabs,
    grow,
    inside,
    type Label,
    LINE,
    lines,
    local as boxOf,
    move,
    type Pt,
    read,
    resize,
    rot,
    round,
    thin,
    tip,
    tiny,
    type Tool,
    turn,
    write,
  } from './scene'
  import Shapes, { tones } from './Shapes.svelte'

  let { note, solo = false }: { note: Note; solo?: boolean } = $props()

  const tools: { t: Tool; icon: Name; key: string; label: string }[] = [
    { t: 'select', icon: 'cursor', key: 'v', label: 'Selecionar' },
    { t: 'hand', icon: 'hand', key: 'h', label: 'Mover a vista' },
    { t: 'pen', icon: 'pen', key: 'p', label: 'Caneta' },
    { t: 'rect', icon: 'square', key: 'r', label: 'Retângulo' },
    { t: 'ellipse', icon: 'circle', key: 'o', label: 'Elipse' },
    { t: 'arrow', icon: 'arrow', key: 'a', label: 'Seta' },
    { t: 'text', icon: 'type', key: 't', label: 'Texto' },
  ]
  const DEPTH = 100
  const HIT = 8
  const ZMIN = 0.1
  const ZMAX = 4
  const STEP = Math.PI / 12
  const KINDS = ['pen', 'rect', 'ellipse', 'image', 'arrow', 'text']

  type Cam = { x: number; y: number; z: number }
  type Grab = { orig: El[]; ids: Set<string>; moved: boolean }
  type Grip =
    | { k: 'pan'; sx: number; sy: number; cx: number; cy: number }
    | { k: 'box'; x: number; y: number; base: string[] }
    | ({ k: 'drag'; x: number; y: number } & Grab)
    | ({ k: 'size'; i: number; area: Box } & Grab)
    | ({ k: 'spin'; c: Pt; a0: number; r0: number } & Grab)
    | ({ k: 'tip'; end: 1 | 2 } & Grab)
    | { k: 'draw'; x: number; y: number }
    | { k: 'pinch'; d: number; m: Pt; cam: Cam }
  type Handle = { k: 'size'; i: number } | { k: 'spin' } | { k: 'tip'; end: 1 | 2 }

  let els = $derived(read(note.body))
  let title = $derived(note.title)
  let tool = $state<Tool>('select')
  let tone = $state(0)
  let sel = $state<string[]>([])
  let draft = $state<El | null>(null)
  let editing = $state<Label | null>(null)
  let area = $state<Box | null>(null)
  let cam = $state<Cam>({ x: 0, y: 0, z: 1 })
  let space = $state(false)
  let panning = $state(false)
  let hot = $state('')
  let palette = $state(false)
  let depth = $state(0)
  let past = $state<string[]>([])
  let future = $state<string[]>([])
  let svg: SVGSVGElement
  let file: HTMLInputElement
  let ed = $state<HTMLTextAreaElement>()
  let grip: Grip | null = null
  let hover: Pt | null = null
  let last: Pt | null = null
  const fingers = new Map<number, Pt>()

  const shown = $derived(draft ? [...els, draft] : els)
  const picked = $derived(els.filter((e) => sel.includes(e.id)))
  const outline = $derived.by(() => {
    if (!picked.length || (picked.length === 1 && picked[0].t === 'arrow')) return null
    if (picked.length === 1) return { c: corners(picked[0]), a: angle(picked[0]), m: center(picked[0]) }
    const b = bounds(picked)!
    const c = [
      { x: b.x, y: b.y },
      { x: b.x + b.w, y: b.y },
      { x: b.x + b.w, y: b.y + b.h },
      { x: b.x, y: b.y + b.h },
    ]
    return { c, a: 0, m: { x: b.x + b.w / 2, y: b.y + b.h / 2 } }
  })
  const knob = $derived.by(() => {
    if (!outline) return null
    const [a, b] = outline.c
    const top = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
    const up = rot({ x: 0, y: -26 / cam.z }, { x: 0, y: 0 }, outline.a)
    return { top, x: top.x + up.x, y: top.y + up.y }
  })
  const ends = $derived(picked.length === 1 && picked[0].t === 'arrow' ? picked[0] : null)
  const cursor = $derived(
    panning ? 'grabbing' : space || tool === 'hand' ? 'grab' : tool === 'select' ? hot || 'default' : tool === 'text' ? 'text' : 'crosshair',
  )

  onMount(() => {
    fit()
    svg.addEventListener('wheel', wheel, { passive: false })
    addEventListener('keydown', key, true)
    addEventListener('keyup', release)
    document.addEventListener('copy', copy)
    document.addEventListener('cut', cut)
    document.addEventListener('paste', paste)
    return () => {
      svg.removeEventListener('wheel', wheel)
      removeEventListener('keydown', key, true)
      removeEventListener('keyup', release)
      document.removeEventListener('copy', copy)
      document.removeEventListener('cut', cut)
      document.removeEventListener('paste', paste)
      sweep()
    }
  })

  function sweep() {
    const n = store.find(note.id)
    if (!n) return
    const used = new Set(read(n.body).flatMap((e) => (e.t === 'image' ? [e.img] : [])))
    for (const i of n.images) if (!used.has(i.id)) store.detach(n.id, i.id)
  }

  function fit() {
    const r = svg.getBoundingClientRect()
    if (!r.width) return void requestAnimationFrame(fit)
    const b = bounds(els)
    if (!b) return void (cam = { x: r.width / 2, y: r.height / 2, z: 1 })
    const mx = Math.min(80, r.width * 0.06)
    const z = clamp(Math.min(1, (r.width - mx * 2) / Math.max(b.w, 1), (r.height - 200) / Math.max(b.h, 1)))
    cam = { x: r.width / 2 - (b.x + b.w / 2) * z, y: r.height / 2 - (b.y + b.h / 2) * z, z }
  }

  function clamp(z: number) {
    return Math.min(ZMAX, Math.max(ZMIN, z))
  }

  function local(e: { clientX: number; clientY: number }): Pt {
    const r = svg.getBoundingClientRect()
    return { x: e.clientX - r.left, y: e.clientY - r.top }
  }

  function scene(p: Pt): Pt {
    return { x: (p.x - cam.x) / cam.z, y: (p.y - cam.y) / cam.z }
  }

  function pt(e: { clientX: number; clientY: number }): Pt {
    return scene(local(e))
  }

  function spot(): Pt {
    if (hover) return scene(hover)
    const r = svg.getBoundingClientRect()
    return scene({ x: r.width / 2, y: r.height / 2 })
  }

  function save(next: El[]) {
    past = [...past, note.body].slice(-DEPTH)
    future = []
    put(next)
  }

  function put(next: El[]) {
    els = next
    store.edit(note.id, { body: write(next) })
    const live = new Set(next.map((e) => e.id))
    if (sel.some((id) => !live.has(id))) sel = sel.filter((id) => live.has(id))
  }

  function undo() {
    const b = past.at(-1)
    if (b === undefined) return
    past = past.slice(0, -1)
    future = [note.body, ...future]
    put(read(b))
  }

  function redo() {
    const [b, ...rest] = future
    if (b === undefined) return
    future = rest
    past = [...past, note.body]
    put(read(b))
  }

  function pick(t: Tool) {
    finish()
    tool = t
    if (t !== 'select') sel = []
  }

  function paint(i: number) {
    if (matchMedia('(max-width: 640px)').matches && (i === tone || !palette)) {
      palette = !palette
      if (i === tone) return
    }
    palette = false
    tone = i
    if (sel.length) save(els.map((e) => (sel.includes(e.id) ? { ...e, c: i } : e)))
  }

  function remove() {
    if (sel.length) save(els.filter((e) => !sel.includes(e.id)))
  }

  async function edit(l: Label) {
    sel = []
    editing = { ...l }
    await tick()
    ed?.focus()
  }

  function finish() {
    if (!editing) return
    const l = round({ ...editing, s: editing.s.replace(/\s+$/, '') }) as Label
    editing = null
    tool = 'select'
    const old = els.find((e) => e.id === l.id)
    if (tiny(l)) {
      if (old) save(els.filter((e) => e.id !== l.id))
    } else if (!old) {
      save([...els, l])
    } else if (old.t === 'text' && old.s !== l.s) {
      save(els.map((e) => (e.id === l.id ? l : e)))
    }
  }

  function fresh(p: Pt): Label {
    return { id: uid(), t: 'text', c: tone, x: p.x, y: p.y - FONT * 0.65, s: '' }
  }

  function handle(p: Pt, touch: boolean): Handle | null {
    const r = (touch ? 18 : 9) / cam.z
    const near = (q: Pt) => Math.hypot(p.x - q.x, p.y - q.y) <= r
    if (knob && near(knob)) return { k: 'spin' }
    const i = outline?.c.findIndex(near) ?? -1
    if (i >= 0) return { k: 'size', i }
    if (ends && near({ x: ends.x1, y: ends.y1 })) return { k: 'tip', end: 1 }
    if (ends && near({ x: ends.x2, y: ends.y2 })) return { k: 'tip', end: 2 }
    return null
  }

  function held(p: Pt) {
    const tol = HIT / cam.z
    if (picked.length === 1) return grabs(picked[0], p.x, p.y, tol)
    const b = picked.length ? bounds(picked) : null
    return !!b && p.x >= b.x && p.x <= b.x + b.w && p.y >= b.y && p.y <= b.y + b.h
  }

  function grab(): Grab {
    return { orig: els, ids: new Set(sel), moved: false }
  }

  function pan(e: PointerEvent) {
    grip = { k: 'pan', sx: e.clientX, sy: e.clientY, cx: cam.x, cy: cam.y }
    panning = true
  }

  function down(e: PointerEvent) {
    finish()
    palette = false
    svg.setPointerCapture(e.pointerId)
    fingers.set(e.pointerId, local(e))
    if (fingers.size === 2) return pinchStart()
    if (fingers.size > 2) return
    const p = pt(e)
    if (e.button === 1 || space || tool === 'hand') return pan(e)
    if (e.button !== 0) return
    if (tool === 'select') return choose(e, p)
    if (tool === 'text') {
      e.preventDefault()
      const h = at(els, p.x, p.y, HIT / cam.z)
      void edit(h?.t === 'text' ? h : fresh(p))
      return
    }
    const id = uid()
    if (tool === 'pen') draft = { id, t: 'pen', c: tone, pts: [p.x, p.y] }
    else if (tool === 'arrow') draft = { id, t: 'arrow', c: tone, x1: p.x, y1: p.y, x2: p.x, y2: p.y }
    else draft = { id, t: tool, c: tone, x: p.x, y: p.y, w: 0, h: 0 }
    grip = { k: 'draw', x: p.x, y: p.y }
  }

  function choose(e: PointerEvent, p: Pt) {
    const g = handle(p, e.pointerType !== 'mouse')
    if (g?.k === 'size') {
      grip = { k: 'size', i: g.i, area: bounds(picked)!, ...grab() }
      return
    }
    if (g?.k === 'spin' && outline) {
      grip = { k: 'spin', c: outline.m, a0: Math.atan2(p.y - outline.m.y, p.x - outline.m.x), r0: outline.a, ...grab() }
      return
    }
    if (g?.k === 'tip') {
      grip = { k: 'tip', end: g.end, ...grab() }
      return
    }
    const h = at(els, p.x, p.y, HIT / cam.z)
    if (e.shiftKey) {
      if (h) sel = sel.includes(h.id) ? sel.filter((id) => id !== h.id) : [...sel, h.id]
      else grip = { k: 'box', x: p.x, y: p.y, base: sel }
      return
    }
    if (h && !sel.includes(h.id)) {
      sel = [h.id]
      tone = h.c
    } else if (!h && !held(p)) {
      sel = []
      grip = { k: 'box', x: p.x, y: p.y, base: [] }
      return
    }
    grip = { k: 'drag', x: p.x, y: p.y, ...grab() }
  }

  function over(e: PointerEvent) {
    hover = local(e)
    if (!fingers.has(e.pointerId) || !grip) {
      if (tool === 'select' && e.pointerType === 'mouse') hot = aim(pt(e))
      return
    }
    fingers.set(e.pointerId, hover)
    const g = grip
    if (g.k === 'pinch') return fingers.size === 2 && pinch(g)
    if (g.k === 'pan') return void (cam = { ...cam, x: g.cx + e.clientX - g.sx, y: g.cy + e.clientY - g.sy })
    const p = pt(e)
    if (g.k === 'box') {
      area = frame(g.x, g.y, p.x, p.y)
      const got = inside(els, area).map((x) => x.id)
      sel = [...new Set([...g.base, ...got])]
    } else if (g.k === 'drag') {
      shift(g, (x) => move(x, p.x - g.x, p.y - g.y))
    } else if (g.k === 'size') {
      if (g.ids.size === 1) shift(g, (x) => resize(x, g.i, p, e.shiftKey))
      else {
        const sx = g.i === 1 || g.i === 2 ? 1 : -1
        const sy = g.i >= 2 ? 1 : -1
        const b = g.area
        const f = { x: sx > 0 ? b.x : b.x + b.w, y: sy > 0 ? b.y : b.y + b.h }
        const s = Math.max(0.05, (sx * (p.x - f.x)) / Math.max(b.w, 1), (sy * (p.y - f.y)) / Math.max(b.h, 1))
        shift(g, (x) => grow(x, s, f))
      }
    } else if (g.k === 'spin') {
      let d = Math.atan2(p.y - g.c.y, p.x - g.c.x) - g.a0
      if (e.shiftKey) d = Math.round((g.r0 + d) / STEP) * STEP - g.r0
      shift(g, (x) => turn(x, d, g.c))
    } else if (g.k === 'tip') {
      shift(g, (x) => (x.t === 'arrow' ? tip(x, g.end, p) : x))
    } else {
      draw(e, g, p)
    }
  }

  function shift(g: Grab, f: (e: El) => El) {
    els = g.orig.map((x) => (g.ids.has(x.id) ? f(x) : x))
    g.moved = true
  }

  function draw(e: PointerEvent, g: { x: number; y: number }, p: Pt) {
    const d = draft
    if (!d) return
    if (d.t === 'pen') {
      for (const c of e.getCoalescedEvents?.() ?? [e]) {
        const q = pt(c)
        d.pts.push(q.x, q.y)
      }
    } else if (d.t === 'arrow') {
      d.x2 = p.x
      d.y2 = p.y
    } else if (d.t === 'rect' || d.t === 'ellipse') {
      let { x, y } = p
      if (e.shiftKey) {
        const s = Math.max(Math.abs(x - g.x), Math.abs(y - g.y))
        x = g.x + Math.sign(x - g.x) * s
        y = g.y + Math.sign(y - g.y) * s
      }
      Object.assign(d, frame(g.x, g.y, x, y))
    }
  }

  function aim(p: Pt): string {
    const h = handle(p, false)
    if (h?.k === 'spin') return 'grab'
    if (h?.k === 'size') return h.i % 2 ? 'nesw-resize' : 'nwse-resize'
    if (h?.k === 'tip') return 'crosshair'
    return held(p) || at(els, p.x, p.y, HIT / cam.z) ? 'move' : ''
  }

  function up(e: PointerEvent) {
    fingers.delete(e.pointerId)
    const g = grip
    if (g?.k === 'pinch') {
      if (!fingers.size) grip = null
      return
    }
    grip = null
    panning = false
    area = null
    if (!g) return
    if ('ids' in g && g.moved) save(els.map((x) => (g.ids.has(x.id) ? round(x) : x)))
    if (g.k === 'draw' && draft) {
      const s = $state.snapshot(draft) as El
      draft = null
      const d = s.t === 'pen' ? { ...s, pts: thin(s.pts) } : round(s)
      if (tiny(d)) return
      save([...els, d])
      if (tool !== 'pen') {
        tool = 'select'
        sel = [d.id]
      }
    }
  }

  function pinchStart() {
    draft = null
    panning = false
    area = null
    els = read(note.body)
    const [a, b] = [...fingers.values()]
    grip = { k: 'pinch', d: Math.hypot(a.x - b.x, a.y - b.y) || 1, m: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }, cam: { ...cam } }
  }

  function pinch(g: Extract<Grip, { k: 'pinch' }>) {
    const [a, b] = [...fingers.values()]
    const z = clamp((g.cam.z * Math.hypot(a.x - b.x, a.y - b.y)) / g.d)
    const sx = (g.m.x - g.cam.x) / g.cam.z
    const sy = (g.m.y - g.cam.y) / g.cam.z
    cam = { z, x: (a.x + b.x) / 2 - sx * z, y: (a.y + b.y) / 2 - sy * z }
  }

  function wheel(e: WheelEvent) {
    e.preventDefault()
    if (!e.ctrlKey && !e.metaKey) return void (cam = { ...cam, x: cam.x - e.deltaX, y: cam.y - e.deltaY })
    const m = local(e)
    const z = clamp(cam.z * Math.exp(-Math.max(-30, Math.min(30, e.deltaY)) * 0.01))
    cam = { z, x: m.x - ((m.x - cam.x) * z) / cam.z, y: m.y - ((m.y - cam.y) * z) / cam.z }
  }

  function zoom(f: number) {
    const r = svg.getBoundingClientRect()
    const z = clamp(cam.z * f)
    const m = { x: r.width / 2, y: r.height / 2 }
    cam = { z, x: m.x - ((m.x - cam.x) * z) / cam.z, y: m.y - ((m.y - cam.y) * z) / cam.z }
  }

  function dbl(e: MouseEvent) {
    if (tool !== 'select') return
    const p = pt(e)
    const h = at(els, p.x, p.y, HIT / cam.z)
    if (!h || h.t === 'text') void edit(h?.t === 'text' ? h : fresh(p))
  }

  async function place(files: File[], p: Pt) {
    const pics = files.filter((f) => f.type.startsWith('image/')).slice(0, 10)
    const r = svg.getBoundingClientRect()
    const max = Math.min(480, Math.min(r.width, r.height) * 0.6) / cam.z
    const made: El[] = []
    for (const [i, f] of pics.entries()) {
      const img = await store.upload(note.id, f)
      if (!img) continue
      const k = Math.min(1 / cam.z, max / Math.max(img.w || max, img.h || max))
      const w = (img.w || max) * k
      const h = (img.h || max * 0.75) * k
      const o = (i * 24) / cam.z
      made.push(round({ id: uid(), t: 'image', c: 0, img: img.id, x: p.x - w / 2 + o, y: p.y - h / 2 + o, w, h }))
    }
    if (!made.length) return
    save([...els, ...made])
    tool = 'select'
    sel = made.map((e) => e.id)
  }

  function writing(t: EventTarget | null) {
    return t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA)$/.test(t.tagName))
  }

  function copy(e: ClipboardEvent) {
    if (writing(e.target) || !picked.length || !e.clipboardData) return false
    e.preventDefault()
    e.clipboardData.setData('text/plain', JSON.stringify({ mindblow: 'board', els: picked }))
    last = null
    return true
  }

  function cut(e: ClipboardEvent) {
    if (copy(e)) remove()
  }

  function parse(t: string): El[] | null {
    try {
      const v = JSON.parse(t)
      if (v?.mindblow !== 'board' || !Array.isArray(v.els)) return null
      return v.els.filter((x: El) => x && typeof x.id === 'string' && KINDS.includes(x.t))
    } catch {
      return null
    }
  }

  function paste(e: ClipboardEvent) {
    if (writing(e.target) || !e.clipboardData) return
    const files = [...e.clipboardData.files].filter((f) => f.type.startsWith('image/'))
    if (files.length) {
      e.preventDefault()
      return void place(files, spot())
    }
    const t = e.clipboardData.getData('text/plain')
    if (!t.trim()) return
    e.preventDefault()
    const got = parse(t)
    void (got ? drop(got) : jot(t))
  }

  async function drop(got: El[]) {
    const b = bounds(got)
    if (!b) return
    let p = spot()
    if (last && Math.hypot(p.x - last.x, p.y - last.y) < 2 / cam.z) p = { x: last.x + 20 / cam.z, y: last.y + 20 / cam.z }
    last = p
    const mine = new Set(note.images.map((i) => i.id))
    const out: El[] = []
    for (const x of clone(got, p.x - (b.x + b.w / 2), p.y - (b.y + b.h / 2), uid)) {
      if (x.t !== 'image' || mine.has(x.img)) {
        out.push(x)
        continue
      }
      const img = await fetch(src(x.img))
        .then((r) => (r.ok ? r.blob() : Promise.reject()))
        .then((bl) => store.upload(note.id, new File([bl], 'imagem', { type: bl.type })))
        .catch(() => undefined)
      if (img) out.push({ ...x, img: img.id })
    }
    if (!out.length) return
    save([...els, ...out.map(round)])
    tool = 'select'
    sel = out.map((x) => x.id)
  }

  function jot(t: string) {
    const p = spot()
    const s = t.replace(/\r\n/g, '\n').trimEnd().slice(0, 5000)
    const l: Label = { id: uid(), t: 'text', c: tone, x: p.x, y: p.y, s }
    const b = boxOf(l)
    const n = round({ ...l, x: p.x - b.w / 2, y: p.y - b.h / 2 })
    save([...els, n])
    tool = 'select'
    sel = [n.id]
  }

  function files(e: DragEvent) {
    return [...(e.dataTransfer?.types ?? [])].includes('Files')
  }

  function key(e: KeyboardEvent) {
    if (writing(e.target)) return
    const mod = e.ctrlKey || e.metaKey
    const k = e.key.toLowerCase()
    if (mod && k === 'z') {
      e.preventDefault()
      return e.shiftKey ? redo() : undo()
    }
    if (mod && k === 'y') {
      e.preventDefault()
      return redo()
    }
    if (mod && k === 'a') {
      e.preventDefault()
      tool = 'select'
      sel = els.map((x) => x.id)
      return
    }
    if (mod || e.altKey) return
    if (e.key === ' ') {
      e.preventDefault()
      space = true
    } else if ((e.key === 'Delete' || e.key === 'Backspace') && sel.length) {
      e.preventDefault()
      remove()
    } else if (e.key === 'Escape' && (sel.length || tool !== 'select')) {
      e.stopPropagation()
      sel = []
      tool = 'select'
    } else if (k === '0') {
      fit()
    } else {
      const f = tools.find((x) => x.key === k)
      if (f) pick(f.t)
    }
  }

  function release(e: KeyboardEvent) {
    if (e.key === ' ') space = false
  }

  function points(ps: Pt[]) {
    return ps.map((p) => `${p.x},${p.y}`).join(' ')
  }
</script>

<div
  class="board"
  class:solo
  role="region"
  aria-label="Board"
  style:--z={cam.z}
  style:--cx="{cam.x}px"
  style:--cy="{cam.y}px"
  ondragenter={(e) => {
    if (!files(e)) return
    e.preventDefault()
    depth++
  }}
  ondragover={(e) => files(e) && e.preventDefault()}
  ondragleave={() => (depth = Math.max(0, depth - 1))}
  ondrop={(e) => {
    if (!files(e)) return
    e.preventDefault()
    depth = 0
    void place([...(e.dataTransfer?.files ?? [])], pt(e))
  }}
>
  <svg
    bind:this={svg}
    style:cursor
    role="application"
    aria-label="Quadro"
    onpointerdown={down}
    onpointermove={over}
    onpointerup={up}
    onpointercancel={up}
    onpointerleave={() => (hover = null)}
    ondblclick={dbl}
  >
    <g transform="translate({cam.x} {cam.y}) scale({cam.z})">
      <Shapes els={shown} hide={editing?.id ?? null} />
      {#if !editing}
        {#if picked.length > 1}
          {#each picked as e (e.id)}
            <polygon class="each" points={points(corners(e))} />
          {/each}
        {/if}
        {#if outline && knob}
          <polygon class="sel" points={points(outline.c)} />
          <line class="sel" x1={knob.top.x} y1={knob.top.y} x2={knob.x} y2={knob.y} />
          <circle class="grip" cx={knob.x} cy={knob.y} r={5 / cam.z} />
          {#each outline.c as q, i (i)}
            <rect class="grip" x={q.x - 4.5 / cam.z} y={q.y - 4.5 / cam.z} width={9 / cam.z} height={9 / cam.z} rx={2 / cam.z} />
          {/each}
        {/if}
        {#if ends}
          <circle class="grip" cx={ends.x1} cy={ends.y1} r={5 / cam.z} />
          <circle class="grip" cx={ends.x2} cy={ends.y2} r={5 / cam.z} />
        {/if}
      {/if}
      {#if area}
        <rect class="area" x={area.x} y={area.y} width={area.w} height={area.h} />
      {/if}
    </g>
  </svg>

  {#if editing}
    {@const b = boxOf(editing)}
    <textarea
      class="ed"
      bind:this={ed}
      bind:value={editing.s}
      spellcheck="false"
      style:left="{editing.x * cam.z + cam.x}px"
      style:top="{editing.y * cam.z + cam.y}px"
      style:font-size="{font(editing) * cam.z}px"
      style:line-height={LINE}
      style:color={tones[editing.c] ?? tones[0]}
      style:width="{Math.max(4, ...lines(editing.s).map((l) => l.length)) + 2}ch"
      style:height="{lines(editing.s).length * LINE}em"
      style:transform-origin="{(b.w * cam.z) / 2}px {(b.h * cam.z) / 2}px"
      style:rotate="{angle(editing)}rad"
      onblur={finish}
      onkeydown={(e) => {
        if (e.key === 'Escape' || (e.key === 'Enter' && (e.ctrlKey || e.metaKey))) {
          e.preventDefault()
          e.stopPropagation()
          finish()
        }
      }}
    ></textarea>
  {/if}

  {#if !shown.length && !editing}
    <p class="hint">desenhe, cole (ctrl + v) ou solte uma imagem · espaço + arrastar move a vista · ctrl + roda dá zoom</p>
  {/if}

  {#if depth}
    <div class="drop"><span>solte para pôr no board</span></div>
  {/if}

  <header class="head">
    <input
      class="title"
      maxlength="200"
      placeholder="Sem título"
      value={title}
      oninput={(e) => store.edit(note.id, { title: (title = e.currentTarget.value) })}
      onkeydown={(e) => e.key === 'Enter' && e.currentTarget.blur()}
    />
    {#if !solo}
      <Meta {note} icon="board" label="board">{els.length} {els.length === 1 ? 'elemento' : 'elementos'}</Meta>
    {/if}
  </header>

  <div class="bar" class:open={palette} role="toolbar" aria-label="Ferramentas do board">
    {#each tools as t (t.t)}
      <button
        class="ib {t.t}"
        class:on={tool === t.t}
        title="{t.label}  {t.key.toUpperCase()}"
        aria-label={t.label}
        aria-pressed={tool === t.t}
        onclick={() => pick(t.t)}><Icon name={t.icon} /></button
      >
    {/each}
    <input
      type="file"
      accept="image/*"
      multiple
      hidden
      bind:this={file}
      onchange={(e) => {
        void place([...(e.currentTarget.files ?? [])], spot())
        e.currentTarget.value = ''
      }}
    />
    <button
      class="ib"
      title="Imagem (ou cole com Ctrl+V)"
      aria-label="Pôr imagem"
      onclick={() => {
        hover = null
        file.click()
      }}><Icon name="image" /></button
    >
    <i class="sep"></i>
    <div class="tones">
      {#each tones as c, i (i)}
        <button class="tone" class:on={tone === i} style:--c={c} title="Cor {i + 1}" aria-label="Cor {i + 1}" aria-pressed={tone === i} onclick={() => paint(i)}
        ></button>
      {/each}
    </div>
    <i class="sep"></i>
    <button class="ib" title="Desfazer  Ctrl+Z" aria-label="Desfazer" disabled={!past.length} onclick={undo}><Icon name="undo" /></button>
    <button class="ib" title="Refazer  Ctrl+Shift+Z" aria-label="Refazer" disabled={!future.length} onclick={redo}><Icon name="redo" /></button>
    <i class="sep z"></i>
    <button class="ib zoom" title="Enquadrar tudo  0" aria-label="Enquadrar tudo" onclick={fit} ondblclick={() => zoom(1 / cam.z)}
      >{Math.round(cam.z * 100)}%</button
    >
  </div>
</div>

<style>
  .board {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background-image: radial-gradient(circle, var(--line-strong) 1px, transparent 1.2px);
    background-size: calc(24px * var(--z)) calc(24px * var(--z));
    background-position: var(--cx) var(--cy);
  }

  svg {
    display: block;
    width: 100%;
    height: 100%;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }

  .sel,
  .each,
  .area,
  .grip {
    vector-effect: non-scaling-stroke;
    stroke-width: 1.2;
  }

  .sel {
    fill: none;
    stroke: var(--tone-2);
  }

  .each {
    fill: none;
    stroke: var(--tone-2);
    stroke-dasharray: 3 3;
    opacity: 0.6;
  }

  .area {
    fill: color-mix(in srgb, var(--tone-2) 8%, transparent);
    stroke: var(--tone-2);
    stroke-dasharray: 4 3;
  }

  .grip {
    fill: var(--paper);
    stroke: var(--tone-2);
  }

  .ed {
    position: absolute;
    min-width: 2ch;
    padding: 0;
    white-space: pre;
    overflow: hidden;
    caret-color: currentColor;
  }

  .hint {
    position: absolute;
    left: 50%;
    top: 50%;
    width: max-content;
    max-width: calc(100% - 48px);
    margin: 0;
    translate: -50% -50%;
    text-align: center;
    font: 11.5px var(--mono);
    color: var(--faint);
    pointer-events: none;
  }

  .drop {
    position: absolute;
    inset: 12px;
    display: grid;
    place-items: center;
    border: 2px dashed var(--tone-2);
    border-radius: var(--r-xl);
    background: color-mix(in srgb, var(--tone-2) 6%, transparent);
    pointer-events: none;
  }

  .drop span {
    padding: 8px 14px;
    border-radius: var(--r-md);
    background: var(--paper);
    font: 12px var(--mono);
    color: var(--fg);
    box-shadow: var(--shadow-pop);
  }

  .head {
    position: absolute;
    top: 16px;
    left: 20px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-width: min(320px, calc(100% - 40px));
    pointer-events: none;
  }

  .head > * {
    pointer-events: auto;
  }

  .title {
    width: 100%;
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }

  .bar {
    position: absolute;
    top: 14px;
    left: 50%;
    translate: -50% 0;
    z-index: 3;
    display: flex;
    align-items: center;
    gap: 2px;
    max-width: calc(100% - 24px);
    padding: 4px;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    box-shadow: var(--shadow-pop);
  }

  .bar .ib {
    flex: none;
  }

  .bar .ib:disabled {
    opacity: 0.35;
    pointer-events: none;
  }

  .sep {
    flex: none;
    width: 1px;
    height: 18px;
    margin: 0 4px;
    background: var(--line);
  }

  .tones {
    display: flex;
    align-items: center;
  }

  .tone {
    flex: none;
    width: 22px;
    height: 22px;
    margin: 0 2px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--c);
    cursor: pointer;
    transition: box-shadow 120ms;
  }

  .tone.on {
    box-shadow:
      0 0 0 2px var(--paper),
      0 0 0 3.5px var(--c);
  }

  .zoom {
    min-width: 52px;
    font: 11px var(--mono);
  }

  .solo .head {
    top: 12px;
    left: 14px;
  }

  .solo .bar {
    top: auto;
    bottom: 12px;
  }

  @media (max-width: 960px) {
    .bar {
      top: auto;
      bottom: 16px;
    }
  }

  @media (max-width: 640px) {
    .head {
      top: 8px;
    }

    .bar {
      bottom: calc(12px + env(safe-area-inset-bottom));
      gap: 1px;
    }

    .bar .ib {
      min-width: 30px;
      height: 32px;
      padding: 0 4px;
    }

    .tone {
      width: 20px;
      height: 20px;
    }

    .tone:not(.on) {
      display: none;
    }

    .open .tones {
      position: absolute;
      bottom: calc(100% + 8px);
      left: 50%;
      translate: -50% 0;
      gap: 6px;
      padding: 8px 10px;
      background: var(--paper);
      border: 1px solid var(--line);
      border-radius: var(--r-lg);
      box-shadow: var(--shadow-pop);
    }

    .open .tone {
      display: block;
      width: 26px;
      height: 26px;
    }

    .hand,
    .zoom,
    .sep.z {
      display: none;
    }
  }
</style>
