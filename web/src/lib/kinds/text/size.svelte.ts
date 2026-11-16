import { read, write } from '../../local'

const KEY = 'mindblow.text.size'
const steps: Record<string, number> = { '+': 0.1, '=': 0.1, '-': -0.1, _: -0.1 }

let v = $state(read<number>(KEY) ?? 1)

function next(at: number, key: string): number | undefined {
  if (key === '0') return 1
  const d = steps[key]
  if (!d) return undefined
  return Math.min(2, Math.max(0.7, Math.round((at + d) * 10) / 10))
}

export const size = {
  get v() {
    return v
  },
  press(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return
    const n = next(v, e.key)
    if (n === undefined) return
    e.preventDefault()
    v = n
    write(KEY, n)
  },
  sync(e: StorageEvent) {
    if (e.key === KEY) v = read<number>(KEY) ?? 1
  },
}
