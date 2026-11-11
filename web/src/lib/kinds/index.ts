import type { Kind } from '../notes/types'
import type { Spec } from './spec'
import { sticky } from './sticky'
import { text } from './text'
import { todo } from './todo'

export type { Spec } from './spec'

export const kinds: Record<Kind, Spec> = { sticky, text, todo }

export const order: Spec[] = [sticky, text, todo]
