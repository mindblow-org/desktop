import { kinds } from '../kinds'
import type { Shell } from './shell'
import { out } from './out.svelte'

interface Win {
  once(e: string, f: () => void): Promise<unknown>
  setFocus(): Promise<void>
  setAlwaysOnTop(on: boolean): Promise<void>
  minimize(): Promise<void>
  toggleMaximize(): Promise<void>
  close(): Promise<void>
}

interface Tauri {
  webviewWindow: {
    WebviewWindow: { new (label: string, o: object): Win; getByLabel(l: string): Promise<Win | null> }
  }
  window: { getCurrentWindow(): Win }
  event: {
    emitTo(target: string, e: string, p: unknown): Promise<void>
    listen<P>(e: string, f: (e: { payload: P }) => void): Promise<unknown>
  }
}

export function tauri(t: Tauri): Shell {
  const me = () => t.window.getCurrentWindow()
  return {
    can: { detach: true, pin: true, frame: true },
    async detach(n) {
      const label = `n-${n.id}`
      const has = await t.webviewWindow.WebviewWindow.getByLabel(label)
      if (has) return void has.setFocus()
      const [width, height] = kinds[n.kind].win
      const s = kinds[n.kind]
      const w = new t.webviewWindow.WebviewWindow(label, {
        url: `${location.origin}/n/${n.id}`,
        title: s.title(n),
        width,
        height,
        decorations: false,
      })
      out.add(n.id)
      void w.once('tauri://destroyed', () => out.delete(n.id))
    },
    pin: (on) => me().setAlwaysOnTop(on),
    async home(id) {
      await t.event.emitTo('main', 'home', id)
      await me().close()
    },
    onhome(f) {
      void t.event.listen<string>('home', (e) => f(e.payload))
    },
    minimize: () => void me().minimize(),
    maximize: () => void me().toggleMaximize(),
    close: () => void me().close(),
  }
}

export type { Tauri }
