import type { Spec } from '../spec'
import Card from './Card.svelte'
import Full from './Full.svelte'

export const todo: Spec = {
  kind: 'todo',
  label: 'Todo',
  icon: 'todo',
  key: '3',
  ratio: Math.SQRT2,
  win: [420, 580],
  images: 50,
  Card,
  Full,
  empty: (n) => !n.title.trim() && !n.items.length && !n.images.length,
  title: (n) => n.title.trim() || 'Lista',
  foot: () => '',
}
