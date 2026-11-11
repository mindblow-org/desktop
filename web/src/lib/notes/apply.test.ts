import { describe, expect, it } from 'vitest'
import { apply, sort } from './apply'
import type { Note } from './types'

const note = (id: string, o: Partial<Note> = {}): Note => ({
  id,
  kind: 'text',
  title: id,
  body: '',
  items: [],
  images: [],
  pinned: false,
  rank: 0,
  created: 1,
  updated: 1,
  ...o,
})

describe('evento vindo de outro dispositivo', () => {
  it('nota criada lá aparece aqui', () => {
    const l = apply([note('a')], { type: 'created', note: note('b') })
    expect(l.map((n) => n.id)).toEqual(['a', 'b'])
  })

  it('edição mais nova substitui a local', () => {
    const l = apply([note('a', { body: 'velho' })], { type: 'updated', note: note('a', { body: 'novo', updated: 2 }) })
    expect(l[0].body).toBe('novo')
  })

  it('edição atrasada não desfaz a mais nova', () => {
    const l = apply([note('a', { body: 'novo', updated: 5 })], { type: 'updated', note: note('a', { body: 'velho', updated: 3 }) })
    expect(l[0].body).toBe('novo')
  })

  it('nota excluída some', () => {
    expect(apply([note('a'), note('b')], { type: 'deleted', id: 'a' }).map((n) => n.id)).toEqual(['b'])
  })

  it('contagem de dispositivos não mexe nas notas', () => {
    const l = [note('a')]
    expect(apply(l, { type: 'peers', count: 3 })).toBe(l)
  })
})

describe('ordem do grid', () => {
  it('segue o rank', () => {
    expect(sort([note('a', { rank: 2 }), note('b', { rank: -1 }), note('c', { rank: 0 })]).map((n) => n.id)).toEqual(['b', 'c', 'a'])
  })
})

describe('fixadas', () => {
  it('vêm antes das outras, cada grupo na sua ordem', () => {
    const l = sort([note('a', { rank: 0 }), note('b', { rank: 1, pinned: true }), note('c', { rank: -1 }), note('d', { rank: 2, pinned: true })])
    expect(l.map((n) => n.id)).toEqual(['b', 'd', 'c', 'a'])
  })
})
