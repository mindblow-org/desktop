import { describe, expect, it } from 'vitest'
import { better, fit } from './fit'

describe('reduzir foto antes de enviar', () => {
  it('foto de celular cabe em 2048 no lado maior', () => {
    expect(fit(4032, 3024)).toEqual([2048, 1536])
    expect(fit(3024, 4032)).toEqual([1536, 2048])
  })

  it('imagem pequena fica do tamanho que está', () => {
    expect(fit(800, 600)).toEqual([800, 600])
  })

  it('panorama estreito não zera a altura', () => {
    expect(fit(40000, 10)[1]).toBe(1)
  })

  it('GIF vai como está, para não perder a animação', () => {
    expect(better({ type: 'image/gif', size: 5_000_000 }, { size: 10 }, true)).toBe(false)
  })

  it('print pequeno que já é leve fica original', () => {
    expect(better({ type: 'image/png', size: 30_000 }, { size: 45_000 }, false)).toBe(false)
  })

  it('foto grande vira a versão reduzida', () => {
    expect(better({ type: 'image/jpeg', size: 5_000_000 }, { size: 400_000 }, true)).toBe(true)
  })

  it('formato que o servidor não aceita sempre é convertido', () => {
    expect(better({ type: 'image/heic', size: 100 }, { size: 900 }, false)).toBe(true)
  })
})
