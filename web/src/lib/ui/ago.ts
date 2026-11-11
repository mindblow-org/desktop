const MIN = 60_000
const HOUR = 60 * MIN
const DAY = 24 * HOUR

export function ago(t: number, now = Date.now()): string {
  const d = Math.max(0, now - t)
  if (d < MIN) return 'agora'
  if (d < HOUR) return `há ${Math.floor(d / MIN)} min`
  if (d < DAY) return `há ${Math.floor(d / HOUR)} h`
  if (d < 2 * DAY) return 'ontem'
  if (d < 7 * DAY) return `há ${Math.floor(d / DAY)} dias`
  if (d < 35 * DAY) return `há ${Math.floor(d / (7 * DAY))} sem`
  return new Date(t).toLocaleDateString('pt-BR', { day: '2-digit', month: 'short' })
}
