import { describe, expect, it } from 'vitest'
import { at, bounds, center, clone, corners, type El, grabs, grow, inside, move, read, resize, thin, tiny, turn, words, write } from './scene'

const rect: El = { id: 'r', t: 'rect', c: 0, x: 0, y: 0, w: 100, h: 50 }
const note: El = { id: 't', t: 'text', c: 0, x: 200, y: 200, s: 'Fluxo\nde login' }
const line: El = { id: 'a', t: 'arrow', c: 1, x1: 0, y1: 100, x2: 100, y2: 200 }

describe('board', () => {
  it('lê e grava a cena, e board vazio vira corpo vazio', () => {
    expect(read(write([rect, note]))).toEqual([rect, note])
    expect(write([])).toBe('')
    expect(read('')).toEqual([])
    expect(read('rabisco')).toEqual([])
  })

  it('a busca só enxerga os textos', () => {
    expect(words([rect, note])).toBe('Fluxo\nde login')
  })

  it('acerta a borda do retângulo, não o miolo', () => {
    expect(at([rect], 50, 2, 4)?.id).toBe('r')
    expect(at([rect], 50, 25, 4)).toBeUndefined()
  })

  it('o de cima ganha quando dois se sobrepõem', () => {
    const over: El = { ...rect, id: 'r2' }
    expect(at([rect, over], 0, 10, 4)?.id).toBe('r2')
  })

  it('acerta a seta pelo traço', () => {
    expect(at([line], 50, 150, 4)?.id).toBe('a')
    expect(at([line], 80, 120, 4)).toBeUndefined()
  })

  it('texto pega a caixa inteira', () => {
    expect(at([note], 210, 230, 4)?.id).toBe('t')
  })

  it('enquadra todos os elementos', () => {
    expect(bounds([rect, line])).toEqual({ x: 0, y: 0, w: 100, h: 200 })
    expect(bounds([])).toBeNull()
  })

  it('mover leva o traço livre inteiro', () => {
    const pen: El = { id: 'p', t: 'pen', c: 0, pts: [0, 0, 10, 10] }
    expect(move(pen, 5, -5)).toMatchObject({ pts: [5, -5, 15, 5] })
  })

  it('traço livre perde pontos colados mas mantém as pontas', () => {
    expect(thin([0, 0, 0.4, 0.3, 1, 1, 10, 10, 10.6, 10.2])).toEqual([0, 0, 10, 10, 11, 10])
  })

  it('descarta clique sem arrastar e texto vazio', () => {
    expect(tiny({ ...rect, w: 1, h: 2 })).toBe(true)
    expect(tiny({ ...note, s: '  ' })).toBe(true)
    expect(tiny(rect)).toBe(false)
  })

  it('elemento girado acerta o clique no lugar girado', () => {
    const tall: El = { ...rect, r: Math.PI / 2 }
    expect(at([tall], 25, -25, 2)?.id).toBe('r')
    expect(at([tall], 0, 2, 2)).toBeUndefined()
  })

  it('caixa de elemento girado cobre os cantos girados', () => {
    expect(bounds([{ ...rect, r: Math.PI / 2 }])).toEqual(expect.objectContaining({ w: expect.closeTo(50), h: expect.closeTo(100) }))
  })

  it('redimensionar pelo canto deixa o canto oposto parado', () => {
    const r2 = resize(rect, 2, { x: 200, y: 150 }, false)
    expect(r2).toMatchObject({ x: 0, y: 0, w: 200, h: 150 })
  })

  it('redimensionar elemento girado mantém o canto oposto no mesmo lugar', () => {
    const tilt: El = { ...rect, r: 0.6 }
    const before = corners(tilt)[0]
    const after = corners(resize(tilt, 2, { x: 300, y: 300 }, false))[0]
    expect(after.x).toBeCloseTo(before.x)
    expect(after.y).toBeCloseTo(before.y)
  })

  it('texto e imagem mantêm a proporção, e no texto cresce a fonte', () => {
    const big = resize(note, 2, { x: 400, y: 260 }, false)
    expect(big.t === 'text' && big.f! > 20).toBe(true)
    const pic: El = { id: 'i', t: 'image', c: 0, img: 'x', x: 0, y: 0, w: 200, h: 100 }
    expect(resize(pic, 2, { x: 400, y: 120 }, false)).toMatchObject({ w: 400, h: 200 })
  })

  it('girar o grupo gira cada um em volta do centro comum', () => {
    const a = turn(rect, Math.PI, { x: 150, y: 25 })
    expect(center(a).x).toBeCloseTo(250)
    expect(a.r).toBeCloseTo(Math.PI)
    const s = turn(line, Math.PI / 2, { x: 0, y: 100 })
    expect(s).toMatchObject({ x1: 0, y1: 100, x2: expect.closeTo(-100), y2: expect.closeTo(200) })
  })

  it('escalar o grupo afasta do ponto fixo', () => {
    expect(grow(rect, 2, { x: 0, y: 0 })).toMatchObject({ x: 0, y: 0, w: 200, h: 100 })
  })

  it('seleção por retângulo pega só o que cabe inteiro', () => {
    expect(inside([rect, line, note], { x: -10, y: -10, w: 150, h: 100 }).map((e) => e.id)).toEqual(['r'])
  })

  it('com o elemento selecionado, arrastar pega por dentro também', () => {
    expect(grabs(rect, 50, 25, 2)).toBe(true)
    expect(at([rect], 50, 25, 2)).toBeUndefined()
  })

  it('colar gera ids novos e desloca', () => {
    let n = 0
    const [c] = clone([rect], 20, 20, () => `n${n++}`)
    expect(c).toMatchObject({ id: 'n0', x: 20, y: 20 })
  })
})
