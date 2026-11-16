import { describe, expect, it } from 'vitest'
import { uris } from './uris'

describe('arquivos copiados no gerenciador de arquivos', () => {
  it('um print copiado no Nautilus vira imagem png com o nome do arquivo', () => {
    expect(uris('file:///home/v/Imagens/Captura%20de%20tela.png\r\n')).toEqual([
      { uri: 'file:///home/v/Imagens/Captura%20de%20tela.png', name: 'Captura de tela.png', type: 'image/png' },
    ])
  })

  it('várias fotos de uma vez, com extensão em maiúscula', () => {
    const got = uris('file:///fotos/a.JPG\nfile:///fotos/b.jpeg\nfile:///fotos/c.webp')
    expect(got.map((p) => [p.name, p.type])).toEqual([
      ['a.JPG', 'image/jpeg'],
      ['b.jpeg', 'image/jpeg'],
      ['c.webp', 'image/webp'],
    ])
  })

  it('pdf, pasta e link da web junto com a imagem ficam de fora', () => {
    const got = uris('# copiados\nfile:///docs/contrato.pdf\nfile:///docs/pasta\nhttps://site.com/a.png\nfile:///docs/assinatura.png')
    expect(got.map((p) => p.name)).toEqual(['assinatura.png'])
  })

  it('texto comum colado não vira imagem', () => {
    expect(uris('ideia.png pra capa')).toEqual([])
    expect(uris('')).toEqual([])
  })
})
