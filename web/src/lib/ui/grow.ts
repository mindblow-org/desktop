function boxes(el: HTMLElement) {
  const out: HTMLElement[] = []
  for (let p = el.parentElement; p; p = p.parentElement) {
    if (p.scrollTop > 0) out.push(p)
  }
  return out
}

export function fit(el: HTMLTextAreaElement) {
  const run = () => {
    const kept = boxes(el).map((b) => [b, b.scrollTop] as const)
    el.style.height = 'auto'
    el.style.height = `${el.scrollHeight}px`
    for (const [b, top] of kept) b.scrollTop = top
  }
  run()
  el.addEventListener('input', run)
  const ro = new ResizeObserver(run)
  ro.observe(el)
  return () => {
    el.removeEventListener('input', run)
    ro.disconnect()
  }
}
