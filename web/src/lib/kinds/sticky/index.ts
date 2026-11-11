import { line, type Spec } from '../spec'
import Card from './Card.svelte'
import Full from './Full.svelte'

export const sticky: Spec = {
  kind: 'sticky',
  label: 'Sticky',
  icon: 'sticky',
  key: '1',
  ratio: 1,
  win: [320, 340],
  images: 1,
  Card,
  Full,
  empty: (n) => !n.body.trim() && !n.images.length,
  title: (n) => line(n.body) || 'Sticky',
}
