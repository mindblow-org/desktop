import type { Note } from './types'

export function between(a?: number, b?: number): number {
  if (a === undefined && b === undefined) return 0
  if (a === undefined) return b! - 1
  if (b === undefined) return a + 1
  return (a + b) / 2
}

export function place(list: Note[], id: string, at: number): number {
  const rest = list.filter((n) => n.id !== id)
  return between(rest[at - 1]?.rank, rest[at]?.rank)
}
