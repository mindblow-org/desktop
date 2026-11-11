import { describe, expect, it } from 'vitest'
import { guess } from './name'

describe('nome do aparelho', () => {
  it('lê sistema e navegador', () => {
    expect(guess('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0 Safari/537.36', false)).toBe('Linux · Chrome')
    expect(guess('Mozilla/5.0 (iPhone; CPU iPhone OS 19_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/19.0 Mobile/15E148 Safari/604.1', false)).toBe('iPhone · Safari')
    expect(guess('Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0 Safari/537.36 Edg/149.0', false)).toBe('Windows · Edge')
    expect(guess('Mozilla/5.0 (Android 15; Mobile; rv:140.0) Gecko/140.0 Firefox/140.0', false)).toBe('Android · Firefox')
  })

  it('no app Tauri vira app desktop', () => {
    expect(guess('qualquer', true)).toBe('App desktop')
  })
})
