export async function copy(s: string) {
  try {
    await navigator.clipboard.writeText(s)
  } catch {
    const t = document.createElement('textarea')
    t.value = s
    document.body.append(t)
    t.select()
    document.execCommand('copy')
    t.remove()
  }
}
