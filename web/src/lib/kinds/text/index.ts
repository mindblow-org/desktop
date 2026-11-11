import { line, type Spec } from '../spec'
import Card from './Card.svelte'
import Full from './Full.svelte'

export const text: Spec = {
  kind: 'text',
  label: 'Texto',
  icon: 'text',
  key: '2',
  ratio: Math.SQRT2,
  win: [560, 680],
  images: 50,
  Card,
  Full,
  empty: (n) => !n.title.trim() && !n.body.trim(),
  title: (n) => n.title.trim() || line(n.body) || 'Sem título',
}
