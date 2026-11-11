import { page, type Sheet } from './ink'

self.onmessage = (e: MessageEvent<Sheet>) => postMessage(page(e.data))
