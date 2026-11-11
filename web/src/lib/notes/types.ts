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

export type Event =
  | { type: 'created'; note: Note }
  | { type: 'updated'; note: Note }
  | { type: 'deleted'; id: string }
  | { type: 'peers'; count: number }
  | { type: 'resync' }
