export function portal(el: HTMLElement) {
  document.body.appendChild(el)
  return () => el.remove()
}
