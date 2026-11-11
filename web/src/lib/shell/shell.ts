import type { Note } from '../notes/types'

export interface Shell {
  can: { detach: boolean; pin: boolean; frame: boolean }
  detach(n: Note): void
  pin(on: boolean): Promise<void>
  home(id: string): void
  onhome(f: (id: string) => void): void
  minimize(): void
  maximize(): void
  close(): void
}
