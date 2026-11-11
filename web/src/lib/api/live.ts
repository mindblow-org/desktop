import type { Event } from '../notes/types'
import { token } from './token'

type Fn = (e: Event) => void

export class Live {
  #subs = new Set<Fn>()
  #wait = 500
  #state: (on: boolean) => void = () => {}

  constructor(private dev: string) {}

  on(f: Fn) {
    this.#subs.add(f)
    return () => this.#subs.delete(f)
  }

  start(state: (on: boolean) => void) {
    this.#state = state
    this.#open()
  }

  #emit(e: Event) {
    for (const f of this.#subs) f(e)
  }

  #open() {
    const q = new URLSearchParams({ d: this.dev })
    const t = token.get()
    if (t) q.set('token', t)
    const s = location.protocol === 'https:' ? 'wss' : 'ws'
    const ws = new WebSocket(`${s}://${location.host}/api/live?${q}`)
    ws.onopen = () => {
      this.#wait = 500
      this.#state(true)
      this.#emit({ type: 'resync' })
    }
    ws.onmessage = (m) => {
      try {
        this.#emit(JSON.parse(m.data))
      } catch {}
    }
    ws.onclose = () => {
      this.#state(false)
      setTimeout(() => this.#open(), this.#wait)
      this.#wait = Math.min(this.#wait * 2, 10_000)
    }
  }
}
