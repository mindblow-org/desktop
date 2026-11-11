export interface Tabs {
  ids: string[]
  on: string | null
}

export function open(t: Tabs, id: string): Tabs {
  return { ids: t.ids.includes(id) ? t.ids : [...t.ids, id], on: id }
}

export function close(t: Tabs, id: string): Tabs {
  const i = t.ids.indexOf(id)
  if (i < 0) return t
  const ids = t.ids.filter((x) => x !== id)
  const on = t.on !== id ? t.on : (ids[i] ?? ids[i - 1] ?? null)
  return { ids, on }
}

// No celular não há abas: fica só a nota aberta, e voltar ao grid fecha ela.
export function solo(t: Tabs): Tabs {
  const ids = t.on ? [t.on] : []
  return ids.length === t.ids.length && ids[0] === t.ids[0] ? t : { ids, on: t.on }
}

export function prune(t: Tabs, alive: Set<string>): Tabs {
  const ids = t.ids.filter((x) => alive.has(x))
  const on = t.on && alive.has(t.on) ? t.on : null
  return ids.length === t.ids.length && on === t.on ? t : { ids, on }
}
