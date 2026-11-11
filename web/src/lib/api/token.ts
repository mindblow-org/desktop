import { read, write } from '../local'

const KEY = 'mindblow.token'

/** Tira o `token=` do hash e devolve o resto, para `#token=…&new=sticky` ainda criar a nota. */
export function split(hash: string): { token: string | null; rest: string } {
  const parts = hash.replace(/^#/, '').split('&').filter(Boolean)
  const i = parts.findIndex((p) => p.startsWith('token='))
  if (i < 0) return { token: null, rest: hash }
  const token = decodeURIComponent(parts[i].slice(6))
  parts.splice(i, 1)
  return { token, rest: parts.length ? `#${parts.join('&')}` : '' }
}

export const token = {
  get: () => read<string>(KEY) ?? '',
  set: (v: string) => write(KEY, v),
  adopt() {
    const { token, rest } = split(location.hash)
    if (token === null) return
    write(KEY, token)
    history.replaceState(null, '', location.pathname + location.search + rest)
  },
}
