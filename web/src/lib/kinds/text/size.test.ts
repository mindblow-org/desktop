import { beforeEach, describe, expect, it } from 'vitest'
import { size } from './size.svelte'

const ctrl = { ctrlKey: true }

function press(key: string, mods: Partial<KeyboardEvent> = ctrl) {
  let took = false
  size.press({ key, ...mods, preventDefault: () => (took = true) } as KeyboardEvent)
  return took
}

describe('tamanho do texto pelo teclado', () => {
  beforeEach(() => {
    press('0')
  })

  it('ctrl + três vezes deixa o texto em 130%', () => {
    press('+')
    press('=')
    press('+')
    expect(size.v).toBe(1.3)
  })

  it('ctrl - diminui e ctrl 0 volta ao normal', () => {
    press('-')
    press('-')
    expect(size.v).toBe(0.8)
    press('0')
    expect(size.v).toBe(1)
  })

  it('não passa de 200% nem fica menor que 70%', () => {
    for (let i = 0; i < 20; i++) press('+')
    expect(size.v).toBe(2)
    for (let i = 0; i < 20; i++) press('-')
    expect(size.v).toBe(0.7)
  })

  it('cmd no Mac faz o mesmo que ctrl', () => {
    press('+', { metaKey: true })
    expect(size.v).toBe(1.1)
  })

  it('escrever + - 0 na nota não mexe no tamanho nem engole a tecla', () => {
    expect(press('+', {})).toBe(false)
    expect(press('-', {})).toBe(false)
    expect(press('0', {})).toBe(false)
    expect(size.v).toBe(1)
  })

  it('ctrl com outras teclas e ctrl alt + ficam para o resto do app', () => {
    expect(press('k')).toBe(false)
    expect(press('+', { ctrlKey: true, altKey: true })).toBe(false)
    expect(size.v).toBe(1)
  })
})
