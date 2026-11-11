import { describe, expect, it } from 'vitest'
import { sort } from './apply'
import { place } from './rank'
import type { Note } from './types'

const note = (id: string, rank: number): Note => ({ id, kind: 'sticky', title: '', body: id, items: [], images: [], pinned: false, rank, created: 0, updated: 0 })

const drop = (l: Note[], id: string, at: number) =>
  sort(l.map((n) => (n.id === id ? { ...n, rank: place(l, id, at) } : n))).map((n) => n.id)

describe('soltar um card arrastado', () => {
  const l = [note('a', 0), note('b', 1), note('c', 2)]

  it('no começo', () => {
    expect(drop(l, 'c', 0)).toEqual(['c', 'a', 'b'])
  })

  it('no meio', () => {
    expect(drop(l, 'a', 1)).toEqual(['b', 'a', 'c'])
  })

  it('no fim', () => {
    expect(drop(l, 'a', 2)).toEqual(['b', 'c', 'a'])
  })

  it('várias vezes no mesmo vão mantém a ordem', () => {
    let v = l
    for (let i = 0; i < 30; i++) {
      const id = i % 2 ? 'a' : 'c'
      v = v.map((n) => (n.id === id ? { ...n, rank: place(sort(v), id, 1) } : n))
    }
    expect(sort(v).map((n) => n.id)).toEqual(['c', 'a', 'b'])
    expect(new Set(v.map((n) => n.rank)).size).toBe(3)
  })
})
