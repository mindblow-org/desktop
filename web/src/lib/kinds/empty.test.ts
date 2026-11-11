import { describe, expect, it } from 'vitest'
import type { Kind, Note } from '../notes/types'
import { kinds } from '.'

const note = (kind: Kind, o: Partial<Note> = {}): Note => ({
  id: 'a',
  kind,
  title: '',
  body: '',
  items: [],
  images: [],
  pinned: false,
  rank: 0,
  created: 1,
  updated: 1,
  ...o,
})

const pic = { id: 'i', mime: 'image/webp', size: 1, w: 1, h: 1 }

describe('nota vazia some ao voltar para o grid', () => {
  it('sem nada, sticky, texto e todo estão vazios', () => {
    for (const k of ['sticky', 'text', 'todo'] as const) expect(kinds[k].empty(note(k))).toBe(true)
  })

  it('só com imagem não está vazia (a foto compartilhada não pode sumir)', () => {
    for (const k of ['sticky', 'text', 'todo'] as const) expect(kinds[k].empty(note(k, { images: [pic] }))).toBe(false)
  })
})
