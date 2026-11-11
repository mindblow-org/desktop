import { read, write } from '../local'

const KEY = 'mindblow.token'

export const token = {
  get: () => read<string>(KEY) ?? '',
  set: (v: string) => write(KEY, v),
  adopt() {
    const m = /(?:^#|&)token=([^&]*)/.exec(location.hash)
    if (!m) return
    write(KEY, decodeURIComponent(m[1]))
    history.replaceState(null, '', location.pathname + location.search)
  },
}
