import { api, Locked } from '../api/http'
import { Live } from '../api/live'
import { name } from '../devices/name'
import { shrink } from '../images/shrink'
import { device, uid } from '../local'
import { apply, sort } from './apply'
import type { Event, Image, Item, Kind, Note, Patch, Peer } from './types'

const DELAY = 300

export interface Upload {
  key: string
  note: string
  url: string
}

class Store {
  list = $state<Note[]>([])
  peers = $state(0)
  devices = $state<Peer[]>([])
  online = $state(false)
  ready = $state(false)
  locked = $state(false)
  busy = $state(0)
  uploads = $state<Upload[]>([])
  notice = $state('')
  sorted = $derived(sort(this.list))
  #wait = new Map<string, { p: Patch; t: ReturnType<typeof setTimeout> }>()
  #subs = new Set<(e: Event) => void>()
  #live = new Live(device(), name.get)

  start() {
    this.#live.on((e) => this.take(e))
    this.#live.start((on) => (this.online = on))
  }

  on(f: (e: Event) => void) {
    this.#subs.add(f)
    return () => this.#subs.delete(f)
  }

  find(id: string) {
    return this.list.find((n) => n.id === id)
  }

  async load() {
    try {
      this.list = (await api.list()).map((n) => this.#over(n))
      this.locked = false
      this.ready = true
    } catch (e) {
      if (e instanceof Locked) this.locked = true
    }
  }

  take(e: Event) {
    if (e.type === 'peers') {
      this.peers = e.count
      this.devices = e.list ?? []
    } else if (e.type === 'resync') void this.load()
    else if (e.type === 'deleted') this.list = apply(this.list, e)
    else this.list = apply(this.list, { ...e, note: this.#over(e.note) })
    for (const f of this.#subs) f(e)
  }

  async add(kind: Kind) {
    const n = await api.add({ kind })
    this.take({ type: 'created', note: n })
    return n
  }

  edit(id: string, p: Patch) {
    this.#set(id, (n) => ({ ...n, ...p }))
    const w = this.#wait.get(id)
    if (w) clearTimeout(w.t)
    this.#wait.set(id, { p: { ...w?.p, ...p }, t: setTimeout(() => this.flush(id), DELAY) })
    this.busy = this.#wait.size
  }

  async flush(id: string) {
    const w = this.#wait.get(id)
    if (!w) return
    clearTimeout(w.t)
    this.#wait.delete(id)
    this.busy = this.#wait.size
    await this.#run(api.edit(id, w.p))
  }

  flushAll() {
    return Promise.all([...this.#wait.keys()].map((id) => this.flush(id)))
  }

  pin(id: string, on: boolean) {
    this.#set(id, (n) => ({ ...n, pinned: on }))
    void this.#run(api.edit(id, { pinned: on }))
  }

  async attach(id: string, files: File[]) {
    const n = this.find(id)
    if (!n) return
    const pics = files.filter((f) => f.type.startsWith('image/')).slice(0, n.kind === 'sticky' ? 1 : 50)
    if (!pics.length) return this.say('só imagens: png, jpeg, webp ou gif')
    const ups = pics.map((f) => ({ key: uid(), note: n.id, url: URL.createObjectURL(f) }))
    this.uploads = [...this.uploads, ...ups]
    for (const [i, f] of pics.entries()) await this.#send(n.id, f, ups[i])
  }

  upload(id: string, f: File) {
    const up = { key: uid(), note: id, url: URL.createObjectURL(f) }
    this.uploads = [...this.uploads, up]
    return this.#send(id, f, up)
  }

  detach(id: string, img: string) {
    this.#set(id, (n) => ({ ...n, images: n.images.filter((i) => i.id !== img) }))
    void this.#run(api.detach(id, img))
  }

  rename(v: string) {
    name.set(v)
    this.#live.say(name.get())
  }

  say(s: string) {
    this.notice = s
    setTimeout(() => {
      if (this.notice === s) this.notice = ''
    }, 4000)
  }

  async #send(id: string, f: File, up: Upload): Promise<Image | undefined> {
    try {
      const s = await shrink(f)
      const n = this.find(id)
      if (!n) return
      if (n.kind === 'sticky' && n.images[0]) await api.detach(n.id, n.images[0].id)
      const had = new Set(n.images.map((i) => i.id))
      const m = await api.attach(n.id, s.blob, s.w, s.h)
      this.take({ type: 'updated', note: m })
      return m.images.find((i) => !had.has(i.id))
    } catch (e) {
      if (e instanceof Locked) this.locked = true
      this.say(e instanceof Error && e.message ? e.message : 'não deu para enviar a imagem')
    } finally {
      this.uploads = this.uploads.filter((u) => u.key !== up.key)
      URL.revokeObjectURL(up.url)
    }
  }

  move(id: string, rank: number) {
    this.#set(id, (n) => ({ ...n, rank }))
    void this.#run(api.edit(id, { rank }))
  }

  remove(id: string) {
    const w = this.#wait.get(id)
    if (w) clearTimeout(w.t)
    this.#wait.delete(id)
    this.busy = this.#wait.size
    this.take({ type: 'deleted', id })
    void this.#run(api.remove(id))
  }

  push(id: string, i: Item) {
    this.#set(id, (n) => ({ ...n, items: [...n.items, i] }))
    void this.#run(api.push(id, i))
  }

  mark(id: string, item: string, p: Partial<Item>) {
    this.#set(id, (n) => ({ ...n, items: n.items.map((i) => (i.id === item ? { ...i, ...p } : i)) }))
    void this.#run(api.mark(id, item, p))
  }

  pull(id: string, item: string) {
    this.#set(id, (n) => ({ ...n, items: n.items.filter((i) => i.id !== item) }))
    void this.#run(api.pull(id, item))
  }

  #set(id: string, f: (n: Note) => Note) {
    this.list = this.list.map((n) => (n.id === id ? f(n) : n))
  }

  #over(n: Note): Note {
    const w = this.#wait.get(n.id)
    return w ? { ...n, ...w.p } : n
  }

  async #run(p: Promise<Note | void>) {
    try {
      const n = await p
      if (n) this.take({ type: 'updated', note: n })
    } catch (e) {
      if (e instanceof Locked) this.locked = true
      void this.load()
    }
  }
}

export const store = new Store()
