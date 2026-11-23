import { describe, expect, it } from 'vitest'
import { edge } from './edge'

describe('menu do botão direito', () => {
  it('abre no ponto do clique', () => {
    expect(edge(300, 20, 160, 40, 1280, 800)).toEqual({ x: 300, y: 20 })
  })

  it('aba perto da borda direita abre o menu para a esquerda', () => {
    expect(edge(1250, 20, 160, 40, 1280, 800)).toEqual({ x: 1090, y: 20 })
  })

  it('card no pé da tela abre o menu para cima', () => {
    expect(edge(300, 790, 160, 40, 1280, 800)).toEqual({ x: 300, y: 750 })
  })

  it('janela destacada estreita não deixa o menu sair pela esquerda', () => {
    expect(edge(100, 20, 160, 40, 200, 800)).toEqual({ x: 8, y: 20 })
  })
})
