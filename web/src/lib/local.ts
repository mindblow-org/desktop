export function read<T>(key: string): T | undefined {
  try {
    const v = localStorage.getItem(key)
    return v === null ? undefined : (JSON.parse(v) as T)
  } catch {
    return undefined
  }
}

export function write(key: string, v: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(v))
  } catch {}
}

export function uid(): string {
  if (typeof crypto.randomUUID === 'function') return crypto.randomUUID()
  const h = [...Array(32)].map(() => Math.floor(Math.random() * 16).toString(16))
  h[12] = '4'
  h[16] = ((parseInt(h[16], 16) & 3) | 8).toString(16)
  const s = h.join('')
  return `${s.slice(0, 8)}-${s.slice(8, 12)}-${s.slice(12, 16)}-${s.slice(16, 20)}-${s.slice(20)}`
}

export function device(): string {
  const k = 'mindblow.device'
  const v = read<string>(k)
  if (v) return v
  const d = uid()
  write(k, d)
  return d
}
