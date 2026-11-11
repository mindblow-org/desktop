import { line, type Spec } from '../spec'
import Card from './Card.svelte'
import Full from './Full.svelte'
import { read, words } from './scene'

export const board: Spec = {
  kind: 'board',
  label: 'Board',
  icon: 'board',
  key: '4',
  ratio: 0.75,
  win: [760, 540],
  images: 0,
  Card,
  Full,
  empty: (n) => !n.title.trim() && !read(n.body).length,
  title: (n) => n.title.trim() || line(words(read(n.body))) || 'Board',
}
