import { better, fit, KEEP } from './fit'

export interface Shrunk {
  blob: Blob
  w: number
  h: number
}

function encode(c: HTMLCanvasElement, type: string): Promise<Blob> {
  return new Promise((ok, no) => c.toBlob((b) => (b ? ok(b) : no(new Error('não deu para ler a imagem'))), type, 0.85))
}

export async function shrink(f: File): Promise<Shrunk> {
  const bmp = await createImageBitmap(f).catch(() => {
    throw new Error('não deu para ler a imagem')
  })
  const [w, h] = fit(bmp.width, bmp.height)
  const resized = w !== bmp.width || h !== bmp.height
  if (KEEP.includes(f.type)) {
    bmp.close()
    return { blob: f, w: bmp.width, h: bmp.height }
  }
  const c = document.createElement('canvas')
  c.width = w
  c.height = h
  const g = c.getContext('2d')!
  g.drawImage(bmp, 0, 0, w, h)
  bmp.close()
  let blob = await encode(c, 'image/webp')
  if (blob.type !== 'image/webp') {
    g.globalCompositeOperation = 'destination-over'
    g.fillStyle = '#fff'
    g.fillRect(0, 0, w, h)
    blob = await encode(c, 'image/jpeg')
  }
  if (!better(f, blob, resized)) return { blob: f, w, h }
  return { blob, w, h }
}
