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
  Card: Component<{ note: Note }>
  Full: Component<{ note: Note; solo?: boolean }>
  empty(n: Note): boolean
  title(n: Note): string
}

export function line(s: string) {
  return s.trim().split('\n')[0].slice(0, 48)
}
