import { read, write } from '../local'

const KEY = 'mindblow.token'

export const token = {
  get: () => read<string>(KEY) ?? '',
  set: (v: string) => write(KEY, v),
}
