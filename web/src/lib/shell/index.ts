import type { Shell } from './shell'
import { tauri, type Tauri } from './tauri'
import { web } from './web'

export type { Shell } from './shell'

const t = (window as unknown as { __TAURI__?: Tauri }).__TAURI__

export const shell: Shell = t ? tauri(t) : web()
