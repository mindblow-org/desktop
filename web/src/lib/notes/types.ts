export type Kind = 'sticky' | 'text' | 'todo' | 'board'

export interface Item {
  id: string
  text: string
  done: boolean
}

export interface Image {
  id: string
  mime: string
  size: number
  w: number
  h: number
}

export interface Note {
  id: string
  kind: Kind
  title: string
  body: string
  items: Item[]
  images: Image[]
  pinned: boolean
  rank: number
  created: number
  updated: number
}

export type Patch = Partial<Pick<Note, 'title' | 'body' | 'items' | 'pinned' | 'rank'>>

export interface Draft {
  kind: Kind
  title?: string
  body?: string
  items?: Partial<Item>[]
}

export interface Peer {
  id: string
  name: string
  since: number
}

/** Estado do sync com outro aparelho: `wait` ainda não tentou, `off` não respondeu, `old` é um mindblow sem sync, `locked` o token não serve, `self` é este próprio aparelho. */
export type Sync = 'wait' | 'ok' | 'off' | 'old' | 'locked' | 'self' | 'fail'

export interface Link {
  url: string
  token: string
  state: Sync
  /** Último sync que deu certo. */
  at: number | null
  error?: string
}

export type Event =
  | { type: 'created'; note: Note }
  | { type: 'updated'; note: Note }
  | { type: 'deleted'; id: string }
  | { type: 'peers'; count: number; list?: Peer[] }
  | { type: 'resync' }
  | { type: 'sync'; list: Link[] }
