import { describe, expect, it } from 'vitest'
import { close, open, prune } from './tabs'

describe('abas', () => {
  it('abrir uma nota já aberta só foca a aba', () => {
    expect(open({ ids: ['a', 'b'], on: null }, 'a')).toEqual({ ids: ['a', 'b'], on: 'a' })
  })

  it('abrir nota nova põe a aba no fim', () => {
    expect(open({ ids: ['a'], on: 'a' }, 'b')).toEqual({ ids: ['a', 'b'], on: 'b' })
  })

  it('fechar a aba ativa passa para a da direita', () => {
    expect(close({ ids: ['a', 'b', 'c'], on: 'b' }, 'b')).toEqual({ ids: ['a', 'c'], on: 'c' })
  })

  it('fechar a última ativa passa para a da esquerda', () => {
    expect(close({ ids: ['a', 'b'], on: 'b' }, 'b')).toEqual({ ids: ['a'], on: 'a' })
  })

  it('fechar a única volta ao grid', () => {
    expect(close({ ids: ['a'], on: 'a' }, 'a')).toEqual({ ids: [], on: null })
  })

  it('fechar outra aba não muda a ativa', () => {
    expect(close({ ids: ['a', 'b'], on: 'a' }, 'b')).toEqual({ ids: ['a'], on: 'a' })
  })

  it('nota excluída em outro dispositivo fecha a aba', () => {
    expect(prune({ ids: ['a', 'b'], on: 'b' }, new Set(['a']))).toEqual({ ids: ['a'], on: null })
  })

  it('sem mudança devolve o mesmo estado', () => {
    const t = { ids: ['a'], on: 'a' }
    expect(prune(t, new Set(['a']))).toBe(t)
  })
})
