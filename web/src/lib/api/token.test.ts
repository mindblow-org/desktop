import { describe, expect, it } from 'vitest'
import { split } from './token'

describe('token no link', () => {
  it('o QR traz só o token', () => {
    expect(split('#token=a%2Bb')).toEqual({ token: 'a+b', rest: '' })
  })

  it('o app Android manda o token junto com a nota a criar', () => {
    expect(split('#token=x&new=sticky')).toEqual({ token: 'x', rest: '#new=sticky' })
    expect(split('#open=123&token=x')).toEqual({ token: 'x', rest: '#open=123' })
  })

  it('sem token, o hash fica como está', () => {
    expect(split('#new=todo')).toEqual({ token: null, rest: '#new=todo' })
    expect(split('')).toEqual({ token: null, rest: '' })
  })
})
