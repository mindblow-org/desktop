import type { Note } from '../notes/types'

/** Ponto da tela, em px de CSS: onde a aba foi solta. */
export interface At {
  x: number
  y: number
}

export interface Shell {
  can: { detach: boolean; pin: boolean; frame: boolean }
  detach(n: Note, at?: At): void
  pin(on: boolean): Promise<void>
  home(id: string): void
  onhome(f: (id: string) => void): void
  minimize(): void
  maximize(): void
  close(): void
}
