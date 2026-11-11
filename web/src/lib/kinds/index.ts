import type { Kind } from '../notes/types'
import { board } from './board'
import type { Spec } from './spec'
import { sticky } from './sticky'
import { text } from './text'
import { todo } from './todo'

export type { Spec } from './spec'

export const kinds: Record<Kind, Spec> = { sticky, text, todo, board }

export const order: Spec[] = [sticky, text, todo, board]
