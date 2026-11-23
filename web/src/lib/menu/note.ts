import { store } from '../notes/store.svelte'
import { copy } from '../ui/copy'
import type { Item } from './store.svelte'

export function note(id: string): Item[] {
  return [{ label: 'Copiar id', icon: 'copy', run: () => copy(id).then(() => store.say('id copiado')) }]
}
