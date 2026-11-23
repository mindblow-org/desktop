import type { Name } from '../ui/Icon.svelte'

export interface Item {
  label: string
  icon: Name
  run: () => void
}

class Menu {
  at = $state<{ x: number; y: number; items: Item[] } | null>(null)

  show(e: MouseEvent, items: Item[]) {
    if ((e as PointerEvent).pointerType === 'touch') return
    e.preventDefault()
    this.at = { x: e.clientX, y: e.clientY, items }
  }

  hide() {
    this.at = null
  }
}

export const menu = new Menu()
