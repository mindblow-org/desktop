const types: Record<string, string> = {
  png: 'image/png',
  jpg: 'image/jpeg',
  jpeg: 'image/jpeg',
  webp: 'image/webp',
  gif: 'image/gif',
  bmp: 'image/bmp',
  avif: 'image/avif',
}

export interface Pic {
  uri: string
  name: string
  type: string
}

function decode(s: string) {
  try {
    return decodeURIComponent(s)
  } catch {
    return s
  }
}

export function uris(list: string): Pic[] {
  return list
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter((l) => l.startsWith('file://'))
    .flatMap((uri) => {
      const name = decode(uri.slice(uri.lastIndexOf('/') + 1))
      const type = types[name.slice(name.lastIndexOf('.') + 1).toLowerCase()]
      return type ? [{ uri, name, type }] : []
    })
}
