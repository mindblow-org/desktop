export function fit(el: HTMLTextAreaElement) {
  const run = () => {
    el.style.height = 'auto'
    el.style.height = `${el.scrollHeight}px`
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
