import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './http'

type Sent = { url: string; keep: boolean; size: number; done: () => void }

function server() {
  const sent: Sent[] = []
  vi.stubGlobal('fetch', (url: string, o: RequestInit) => {
    const size = o.body instanceof Uint8Array ? o.body.byteLength : 0
    if (o.keepalive && size > 64 * 1024) return Promise.reject(new TypeError('Failed to fetch'))
    return new Promise<Response>((ok) => {
      sent.push({ url, keep: !!o.keepalive, size, done: () => ok(Response.json({ id: 'n1', kind: 'board' })) })
    })
  })
  return sent
}

const scene = (kb: number) => JSON.stringify({ elements: [{ points: 'x'.repeat(kb * 1024) }] })

afterEach(() => vi.unstubAllGlobals())

describe('envio ao servidor', () => {
  it('duas folhas escaneadas viram board sem estourar o keepalive', async () => {
    const sent = server()
    const p = api.add({ kind: 'board', body: scene(100) })
    await vi.waitFor(() => expect(sent).toHaveLength(1))
    expect(sent[0].keep).toBe(false)
    sent[0].done()
    await expect(p).resolves.toMatchObject({ kind: 'board' })
  })

  it('editar um sticky segue com keepalive, para salvar mesmo fechando a aba', async () => {
    const sent = server()
    const p = api.edit('n1', { body: 'comprar pão' })
    await vi.waitFor(() => expect(sent).toHaveLength(1))
    expect(sent[0].keep).toBe(true)
    sent[0].done()
    await p
  })

  it('dois textos médios salvando juntos não passam de 64 KiB em keepalive', async () => {
    const sent = server()
    const a = api.edit('a', { body: 'a'.repeat(40 * 1024) })
    const b = api.edit('b', { body: 'b'.repeat(40 * 1024) })
    await vi.waitFor(() => expect(sent).toHaveLength(2))
    expect(sent.map((s) => s.keep)).toEqual([true, false])
    for (const s of sent) s.done()
    await Promise.all([a, b])
    const c = api.edit('c', { body: 'c'.repeat(40 * 1024) })
    await vi.waitFor(() => expect(sent).toHaveLength(3))
    expect(sent[2].keep).toBe(true)
    sent[2].done()
    await c
  })
})
