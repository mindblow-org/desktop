import type { Shell } from '../shell'
import { uris } from './uris'

export function pasted(e: ClipboardEvent, s: Shell): Promise<File[]> | null {
  const d = e.clipboardData
  if (!d) return null
  const f = [...d.files].filter((x) => x.type.startsWith('image/'))
  if (f.length) {
    e.preventDefault()
    return Promise.resolve(f)
  }
  if (!s.can.clip) return null
  const u = uris(d.getData('text/uri-list'))
  if (u.length) {
    e.preventDefault()
    return Promise.all(
      u.map(async (p) => {
        const b = await s.clip(p.uri)
        return b && new File([b], p.name, { type: p.type })
      }),
    ).then((got) => got.filter((x) => x !== null))
  }
  if (d.types.includes('text/plain')) return null
  return s.pic().then((b) => (b ? [new File([b], 'imagem.png', { type: 'image/png' })] : []))
}
