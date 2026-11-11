export function nodrag(e: DragEvent) {
  const t = e.target
  const el = t instanceof Element ? t : t instanceof Node ? t.parentElement : null
  if (!el?.closest('input, textarea, [contenteditable=""], [contenteditable="true"]')) e.preventDefault()
}
