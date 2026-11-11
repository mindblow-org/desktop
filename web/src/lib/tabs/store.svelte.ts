import { read, write } from '../local'
import { close, open, prune, type Tabs } from './tabs'

const KEY = 'mindblow.tabs'

class TabStore {
  #s = $state<Tabs>(read<Tabs>(KEY) ?? { ids: [], on: null })

  get ids() {
    return this.#s.ids
  }

  get on() {
    return this.#s.on
  }

  open(id: string) {
    this.#put(open(this.#s, id))
  }

  close(id: string) {
    this.#put(close(this.#s, id))
  }

  home() {
    this.#put({ ...this.#s, on: null })
  }

  prune(alive: Set<string>) {
    const t = prune(this.#s, alive)
    if (t !== this.#s) this.#put(t)
  }

  #put(t: Tabs) {
    this.#s = t
    write(KEY, t)
  }
}

export const tabs = new TabStore()
