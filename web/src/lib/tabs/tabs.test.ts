import { describe, expect, it } from 'vitest'
import { close, move, open, prune, solo, spot } from './tabs'

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

describe('celular, uma nota por vez', () => {
  it('abrir outra nota troca a aberta', () => {
    expect(solo(open({ ids: ['a'], on: 'a' }, 'b'))).toEqual({ ids: ['b'], on: 'b' })
  })

  it('voltar ao grid fecha a nota', () => {
    expect(solo({ ids: ['a'], on: null })).toEqual({ ids: [], on: null })
  })

  it('abas guardadas de antes ficam só com a ativa', () => {
    expect(solo({ ids: ['a', 'b', 'c'], on: 'b' })).toEqual({ ids: ['b'], on: 'b' })
  })

  it('sem mudança devolve o mesmo estado', () => {
    const t = { ids: ['a'], on: 'a' }
    expect(solo(t)).toBe(t)
  })

  it('arrastar a primeira aba para o fim muda a ordem e mantém a ativa', () => {
    expect(move({ ids: ['a', 'b', 'c'], on: 'b' }, 'a', 2)).toEqual({ ids: ['b', 'c', 'a'], on: 'b' })
  })

  it('arrastar para a esquerda põe a aba antes das outras', () => {
    expect(move({ ids: ['a', 'b', 'c'], on: 'c' }, 'c', 0)).toEqual({ ids: ['c', 'a', 'b'], on: 'c' })
  })

  it('soltar no mesmo lugar não muda nada', () => {
    const t = { ids: ['a', 'b'], on: 'a' }
    expect(move(t, 'a', 0)).toBe(t)
  })

  it('a aba cai depois das que ficaram à esquerda do meio dela', () => {
    const mids = [50, 150, 250, 350]
    expect(spot(mids, 0, 50)).toBe(0)
    expect(spot(mids, 0, 160)).toBe(1)
    expect(spot(mids, 0, 400)).toBe(3)
    expect(spot(mids, 3, 120)).toBe(1)
    expect(spot(mids, 3, 10)).toBe(0)
  })
})
