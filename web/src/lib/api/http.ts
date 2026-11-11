import type { Draft, Item, Note, Patch } from '../notes/types'
import { token } from './token'

export class Locked extends Error {}

async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const h: Record<string, string> = {}
  const raw = body instanceof Blob
  if (body !== undefined) h['content-type'] = raw ? body.type : 'application/json'
  const t = token.get()
  if (t) h.authorization = `Bearer ${t}`
  const r = await fetch(path, {
    method,
    headers: h,
    body: body === undefined ? undefined : raw ? body : JSON.stringify(body),
    keepalive: method !== 'GET' && !raw,
  })
  if (r.status === 401) throw new Locked()
  if (!r.ok) throw new Error((await r.json().catch(() => null))?.error ?? r.statusText)
  return (r.status === 204 ? undefined : await r.json()) as T
}

const at = (id: string) => `/api/notes/${id}`

export interface Health {
  ok: boolean
  version: string
  link: string | null
}

export const api = {
  health: () => call<Health>('GET', '/api/health'),
  list: () => call<Note[]>('GET', '/api/notes'),
  add: (d: Draft) => call<Note>('POST', '/api/notes', d),
  edit: (id: string, p: Patch) => call<Note>('PATCH', at(id), p),
  remove: (id: string) => call<void>('DELETE', at(id)),
  push: (id: string, i: Item) => call<Note>('POST', `${at(id)}/items`, i),
  mark: (id: string, item: string, p: Partial<Item>) => call<Note>('PATCH', `${at(id)}/items/${item}`, p),
  pull: (id: string, item: string) => call<Note>('DELETE', `${at(id)}/items/${item}`),
  attach: (id: string, b: Blob, w: number, h: number) => call<Note>('POST', `${at(id)}/images?w=${w}&h=${h}`, b),
  detach: (id: string, img: string) => call<Note>('DELETE', `${at(id)}/images/${img}`),
}

export function src(img: string) {
  const t = token.get()
  return `/api/images/${img}${t ? `?token=${encodeURIComponent(t)}` : ''}`
}
