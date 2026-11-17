import { describe, expect, it, vi } from 'vitest'
import type { Shell } from '../shell'
import { pasted } from './pasted'

const png = new Uint8Array([137, 80, 78, 71]).buffer

function shell(linux: boolean) {
  return {
    can: { detach: true, pin: true, frame: true, clip: linux },
    clip: vi.fn(async () => png),
    pic: vi.fn(async () => png),
  } as unknown as Shell & { clip: ReturnType<typeof vi.fn>; pic: ReturnType<typeof vi.fn> }
}

function paste(data: Record<string, string>, files: File[] = []) {
  const types = [...Object.keys(data), ...(files.length ? ['Files'] : [])]
  const e = {
    clipboardData: { files, types, getData: (t: string) => data[t] ?? '' },
    preventDefault: vi.fn(),
  }
  return e as unknown as ClipboardEvent & { preventDefault: ReturnType<typeof vi.fn> }
}

describe('colar imagem na nota', () => {
  it('print no app do Linux com WebKitGTK antigo: o paste chega vazio e a imagem vem do clipboard do sistema', async () => {
    const s = shell(true)
    const got = await pasted(paste({}), s)
    expect(s.pic).toHaveBeenCalledOnce()
    expect(got?.map((f) => [f.name, f.type, f.size])).toEqual([['imagem.png', 'image/png', 4]])
  })

  it('print que o WebKit já entrega como arquivo entra direto, sem pedir ao sistema', async () => {
    const s = shell(true)
    const e = paste({}, [new File([png], 'image.png', { type: 'image/png' })])
    const got = await pasted(e, s)
    expect(e.preventDefault).toHaveBeenCalled()
    expect(s.pic).not.toHaveBeenCalled()
    expect(got?.map((f) => f.name)).toEqual(['image.png'])
  })

  it('png copiado no gerenciador de arquivos vem pelo caminho, e o texto file:// não entra', async () => {
    const s = shell(true)
    const e = paste({ 'text/uri-list': 'file:///home/v/capa.png', 'text/plain': 'file:///home/v/capa.png' })
    const got = await pasted(e, s)
    expect(e.preventDefault).toHaveBeenCalled()
    expect(s.clip).toHaveBeenCalledWith('file:///home/v/capa.png')
    expect(got?.map((f) => [f.name, f.type])).toEqual([['capa.png', 'image/png']])
  })

  it('texto copiado segue para o texto e o clipboard do sistema nem é lido', () => {
    const s = shell(true)
    const e = paste({ 'text/plain': 'lista de compras' })
    expect(pasted(e, s)).toBeNull()
    expect(e.preventDefault).not.toHaveBeenCalled()
    expect(s.pic).not.toHaveBeenCalled()
  })

  it('clipboard sem imagem nenhuma: nada é anexado', async () => {
    const s = shell(true)
    s.pic.mockResolvedValueOnce(null)
    expect(await pasted(paste({}), s)).toEqual([])
  })

  it('no navegador o paste vazio não vira pedido ao sistema', () => {
    const s = shell(false)
    expect(pasted(paste({}), s)).toBeNull()
    expect(s.pic).not.toHaveBeenCalled()
  })
})
