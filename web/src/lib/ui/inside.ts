const gap = 12

export function inside(el: HTMLElement) {
  const run = () => {
    el.style.translate = ''
    const r = el.getBoundingClientRect()
    const over = Math.min(r.left + el.offsetWidth - (innerWidth - gap), r.left - gap)
    if (over > 0) el.style.translate = `${-over}px 0`
  }
  run()
  addEventListener('resize', run)
  return () => removeEventListener('resize', run)
}
