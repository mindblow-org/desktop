import { read, write } from '../local'
import { close, move, open, prune, solo, type Tabs } from './tabs'

const KEY = 'mindblow.tabs'
const narrow = matchMedia('(max-width: 640px)')
const fit = (t: Tabs) => (narrow.matches ? solo(t) : t)

class TabStore {
  #s = $state<Tabs>(fit(read<Tabs>(KEY) ?? { ids: [], on: null }))

  get ids() {
    return this.#s.ids
  }

  get on() {
    return this.#s.on
  }

  open(id: string, at?: number) {
    this.#put(open(this.#s, id, at))
  }

  close(id: string) {
    this.#put(close(this.#s, id))
  }

  move(id: string, to: number) {
    this.#put(move(this.#s, id, to))
  }

  home() {
    this.#put({ ...this.#s, on: null })
  }

  prune(alive: Set<string>) {
    const t = prune(this.#s, alive)
    if (t !== this.#s) this.#put(t)
  }

  #put(t: Tabs) {
    this.#s = fit(t)
    write(KEY, this.#s)
  }
}

export const tabs = new TabStore()
