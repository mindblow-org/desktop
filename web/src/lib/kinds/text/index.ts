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
  empty: (n) => !n.title.trim() && !n.body.trim() && !n.images.length,
  title: (n) => n.title.trim() || line(n.body) || 'Sem título',
  foot(n) {
    const w = n.body.trim() ? n.body.trim().split(/\s+/).length : 0
    return `${w} ${w === 1 ? 'palavra' : 'palavras'}`
  },
}
