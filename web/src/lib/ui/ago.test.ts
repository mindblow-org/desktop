import { describe, expect, it } from 'vitest'
import { ago } from './ago'

const now = Date.UTC(2026, 10, 11, 12)
const min = 60_000

describe('editado há', () => {
  it.each([
    [10_000, 'agora'],
    [2 * min, 'há 2 min'],
    [3 * 60 * min, 'há 3 h'],
    [30 * 60 * min, 'ontem'],
    [3 * 24 * 60 * min, 'há 3 dias'],
    [15 * 24 * 60 * min, 'há 2 sem'],
  ])('%i ms atrás vira "%s"', (d, s) => {
    expect(ago(now - d, now)).toBe(s)
  })
})
