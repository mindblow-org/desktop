import { type El, write } from '../kinds/board/scene'
import { draw, page, type Page, type Sheet, SIDE } from './ink'

export const MAX = 10 // folhas por vez
const WIDTH = 900 // largura de cada folha no board
const GAP = 80 // entre uma folha e a de baixo
const LIMIT = 1_000_000 // o servidor recusa cena maior que 1 MB

// Fotos de folhas viram os elementos de um board, uma folha embaixo da outra.
export async function scan(files: File[]): Promise<El[]> {
  const pages: Page[] = []
  for (const f of files.slice(0, MAX)) pages.push(await read(await sheet(f)))
  for (const eps of [0.9, 1.6, 2.8]) {
    const els: El[] = []
    let y = 0
    for (const p of pages) {
      const k = WIDTH / p.w
      els.push(...draw(p, { eps, k, dx: 0, dy: y }))
      y += p.h * k + GAP
    }
    if (write(els).length <= LIMIT) return els
  }
  throw new Error('a folha tem traço demais para um board')
}

async function sheet(f: File): Promise<Sheet> {
  const bmp = await createImageBitmap(f).catch(() => {
    throw new Error('não deu para ler a imagem')
  })
  const k = Math.min(1, SIDE / Math.max(bmp.width, bmp.height))
  const w = Math.max(1, Math.round(bmp.width * k))
  const h = Math.max(1, Math.round(bmp.height * k))
  const c = document.createElement('canvas')
  c.width = w
  c.height = h
  const g = c.getContext('2d', { willReadFrequently: true })!
  g.fillStyle = '#fff'
  g.fillRect(0, 0, w, h)
  g.drawImage(bmp, 0, 0, w, h)
  bmp.close()
  return { data: g.getImageData(0, 0, w, h).data, w, h }
}

// Num worker, para a tela não travar no celular; sem worker, roda aqui mesmo.
function read(s: Sheet): Promise<Page> {
  let wk: Worker
  try {
    wk = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' })
  } catch {
    return Promise.resolve(page(s))
  }
  return new Promise((ok) => {
    wk.onmessage = (e: MessageEvent<Page>) => {
      wk.terminate()
      ok(e.data)
    }
    wk.onerror = () => {
      wk.terminate()
      ok(page(s))
    }
    wk.postMessage(s)
  })
}
