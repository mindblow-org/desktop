import { describe, expect, it } from 'vitest'
import type { Note } from '../notes/types'
import { fold, mark, search } from './match'

let seq = 0
const note = (o: Partial<Note>): Note => ({
  id: String(seq++),
  kind: 'text',
  title: '',
  body: '',
  items: [],
  images: [],
  pinned: false,
  rank: 0,
  created: 0,
  updated: 0,
  ...o,
})
const name = (n: Note) => n.title || n.body.split('\n')[0]
const titles = (q: string, l: Note[]) => search(l, q, name).map((h) => h.note.title || h.note.body)
const text = (p: { t: string; on: boolean }[]) => p.map((x) => (x.on ? `[${x.t}]` : x.t)).join('')

const notes = [
  note({ title: 'Ideia: mindblow', body: 'Notas que moram numa máquina só', updated: 3 }),
  note({ title: 'Comandos tailscale', body: 'tailscale serve status\nsystemctl --user restart mindblow', updated: 9 }),
  note({ kind: 'todo', title: 'Mercado', items: [{ id: 'i', text: 'Café', done: false }, { id: 'j', text: 'pão', done: true }], updated: 1 }),
  note({ kind: 'sticky', body: 'comprar filtro de café', updated: 5 }),
  note({ title: 'Reunião ABRAZ', body: 'pauta do zarb2', updated: 2 }),
]

describe('busca da paleta', () => {
  it('não liga para acento nem maiúscula', () => {
    expect(titles('CAFE', notes)).toEqual(['comprar filtro de café', 'Mercado'])
    expect(titles('reuniao', notes)).toEqual(['Reunião ABRAZ'])
  })

  it('título vem antes de conteúdo, mesmo que o conteúdo seja mais recente', () => {
    expect(titles('mindblow', notes)).toEqual(['Ideia: mindblow', 'Comandos tailscale'])
  })

  it('todos os termos precisam bater', () => {
    expect(titles('mercado pao', notes)).toEqual(['Mercado'])
    expect(titles('mercado zarb2', notes)).toEqual([])
  })

  it('acha itens do todo', () => {
    expect(titles('pão', notes)).toEqual(['Mercado'])
  })

  it('campo vazio mostra as editadas por último', () => {
    expect(titles('  ', notes).slice(0, 2)).toEqual(['Comandos tailscale', 'comprar filtro de café'])
  })

  it('o trecho mostra onde bateu, com o termo marcado', () => {
    const h = search(notes, 'mindblow', name).find((h) => h.note.title === 'Comandos tailscale')!
    expect(text(h.snip)).toContain('restart [mindblow]')
  })

  it('o título marca o termo sem perder o acento original', () => {
    const h = search(notes, 'reuniao', name)[0]
    expect(text(h.title)).toBe('[Reunião] ABRAZ')
  })

  it('sticky não repete o próprio texto no trecho', () => {
    const h = search(notes, 'filtro', name)[0]
    expect(h.snip).toEqual([])
  })
})

describe('fold', () => {
  it('mantém o tamanho do texto, para o destaque cair no lugar certo', () => {
    const s = 'Ação rápida 🚀 já'
    expect(fold(s).length).toBe(s.length)
    expect(text(mark(s, fold(s), ['ja']))).toBe('Ação rápida 🚀 [já]')
  })

  it('no board só os textos da cena contam', () => {
    const b = note({ kind: 'board', title: 'Arquitetura', body: JSON.stringify({ v: 1, els: [{ id: 'a', t: 'text', c: 0, x: 0, y: 0, s: 'fila de jobs' }] }) })
    expect(titles('jobs', [b])).toEqual(['Arquitetura'])
    expect(titles('els', [b])).toEqual([])
  })
})
