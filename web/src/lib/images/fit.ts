export const EDGE = 2048
export const KEEP = ['image/gif']
export const TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif']

export function fit(w: number, h: number, max = EDGE): [number, number] {
  if (w <= max && h <= max) return [w, h]
  const k = max / Math.max(w, h)
  return [Math.max(1, Math.round(w * k)), Math.max(1, Math.round(h * k))]
}

export function better(orig: { type: string; size: number }, made: { size: number }, resized: boolean): boolean {
  if (KEEP.includes(orig.type)) return false
  if (!TYPES.includes(orig.type)) return true
  return resized || made.size < orig.size
}
