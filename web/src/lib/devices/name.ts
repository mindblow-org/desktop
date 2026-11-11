import { read, write } from '../local'

const KEY = 'mindblow.name'

export function guess(ua: string, app: boolean): string {
  if (app) return 'App desktop'
  const os = /iPhone/.test(ua)
    ? 'iPhone'
    : /iPad/.test(ua)
      ? 'iPad'
      : /Android/.test(ua)
        ? 'Android'
        : /Macintosh|Mac OS X/.test(ua)
          ? 'Mac'
          : /Windows/.test(ua)
            ? 'Windows'
            : /Linux|X11|CrOS/.test(ua)
              ? 'Linux'
              : 'Navegador'
  const browser = /Edg\//.test(ua)
    ? 'Edge'
    : /Firefox\/|FxiOS/.test(ua)
      ? 'Firefox'
      : /OPR\//.test(ua)
        ? 'Opera'
        : /Chrome\/|CriOS/.test(ua)
          ? 'Chrome'
          : /Safari\//.test(ua)
            ? 'Safari'
            : ''
  return browser ? `${os} · ${browser}` : os
}

export const name = {
  get: () => read<string>(KEY) || guess(navigator.userAgent, '__TAURI__' in window),
  set: (v: string) => write(KEY, v.trim().slice(0, 40)),
}
