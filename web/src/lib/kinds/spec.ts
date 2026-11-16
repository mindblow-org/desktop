import type { Component } from 'svelte'
import type { Kind, Note } from '../notes/types'
import type { Name } from '../ui/Icon.svelte'

export interface Spec {
  kind: Kind
  label: string
  icon: Name
  key: string
  ratio: number
  win: [number, number]
  images: number
  Card: Component<{ note: Note }>
  // kit: no celular, se a barra de ferramentas do board está aberta. Os outros tipos ignoram.
  Full: Component<{ note: Note; solo?: boolean; kit?: boolean }>
  empty(n: Note): boolean
  title(n: Note): string
  foot?(n: Note): string
}

export function line(s: string) {
  return s.trim().split('\n')[0].slice(0, 48)
}
