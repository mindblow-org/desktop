import { describe, expect, it } from 'vitest'
import { GAP, HEAD, layout, near, shift, size, stack } from './layout'

describe('masonry', () => {
  it('cabe quantas colunas a largura deixar', () => {
    expect(size(1140).n).toBe(5)
    expect(size(700).n).toBe(3)
  })

  it('no celular usa duas colunas que enchem a tela', () => {
    const s = size(354)
    expect(s.n).toBe(2)
    expect(s.w * 2 + s.g).toBeLessThanOrEqual(354)
  })

  it('cada nota entra na coluna mais baixa', () => {
    const s = { n: 2, w: 100, g: 10 }
    const { boxes } = layout([140, 100, 100], s)
    expect(boxes[2]).toEqual({ x: 110, y: 110, h: 100 })
  })

  it('altura do grid é a da coluna mais alta', () => {
    expect(layout([140, 100, 100], { n: 2, w: 100, g: 10 }).height).toBe(210)
  })

  it('grid vazio não tem altura', () => {
    expect(layout([], { n: 3, w: 100, g: 10 }).height).toBe(0)
  })
})

describe('arrastar', () => {
  const boxes = layout([140, 100, 100], { n: 2, w: 100, g: 10 }).boxes

  it('acha o card sob o ponteiro', () => {
    expect(near(boxes, 160, 160, 100)).toBe(2)
  })

  it('longe de todos não troca nada', () => {
    expect(near(boxes, 900, 900, 100)).toBe(-1)
  })

  it('move o item para a nova posição', () => {
    expect(shift(['a', 'b', 'c', 'd'], 3, 1)).toEqual(['a', 'd', 'b', 'c'])
    expect(shift(['a', 'b', 'c', 'd'], 0, 2)).toEqual(['b', 'c', 'a', 'd'])
  })
})

describe('seções Fixadas e Outras', () => {
  const s = { n: 2, w: 100, g: 10 }

  it('outras começam abaixo das fixadas e do título', () => {
    const m = stack([[100], [140, 100]], s, true)
    expect(m.boxes[0][0].y).toBe(HEAD)
    expect(m.heads[1]).toBe(HEAD + 100 + GAP)
    expect(m.boxes[1][0].y).toBe(HEAD + 100 + GAP + HEAD)
  })

  it('sem fixadas, o grid fica igual ao de antes', () => {
    const m = stack([[], [140, 100]], s, false)
    expect(m.boxes[1]).toEqual(layout([140, 100], s).boxes)
    expect(m.height).toBe(140)
  })

  it('só fixadas não deixa vão sobrando embaixo', () => {
    expect(stack([[100], []], s, true).height).toBe(HEAD + 100)
  })
})
