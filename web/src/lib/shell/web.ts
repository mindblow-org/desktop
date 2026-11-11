import { kinds } from '../kinds'
import type { Shell } from './shell'
import { out } from './out.svelte'

export function web(): Shell {
  const wide = matchMedia('(min-width: 720px) and (pointer: fine)').matches
  return {
    can: { detach: wide, pin: false },
    detach(n) {
      const [w, h] = kinds[n.kind].win
      const win = window.open(`/n/${n.id}`, `n-${n.id}`, `popup,width=${w},height=${h}`)
      if (!win) return
      out.add(n.id)
      const t = setInterval(() => {
        if (!win.closed) return
        clearInterval(t)
        out.delete(n.id)
      }, 800)
    },
    async pin() {},
    home(id) {
      window.opener?.postMessage({ home: id }, location.origin)
      window.close()
    },
    onhome(f) {
      addEventListener('message', (e) => {
        if (e.origin === location.origin && typeof e.data?.home === 'string') f(e.data.home)
      })
    },
  }
}
