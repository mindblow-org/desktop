import type { Event, Note } from './types'

export function apply(list: Note[], e: Event): Note[] {
  switch (e.type) {
    case 'created':
    case 'updated': {
      const i = list.findIndex((n) => n.id === e.note.id)
      if (i < 0) return [...list, e.note]
      if (list[i].updated > e.note.updated) return list
      return list.with(i, e.note)
    }
    case 'deleted':
      return list.filter((n) => n.id !== e.id)
    default:
      return list
  }
}

export function sort(list: Note[]): Note[] {
  return [...list].sort((a, b) => Number(b.pinned) - Number(a.pinned) || a.rank - b.rank || (a.id < b.id ? -1 : 1))
}
