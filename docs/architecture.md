# Arquitetura

```
crates/domain   regras e casos de uso, sem IO
crates/store    Repo em SQLite
crates/server   HTTP, WebSocket e a interface embutida (bin `mindblow`)
app/            janela Tauri que carrega o servidor
web/            interface Svelte 5
deploy/         systemd + tailscale serve
```

Um processo só, numa máquina, guarda tudo. Os dispositivos são clientes: o app Tauri e o navegador/PWA carregam a mesma interface, que o próprio servidor entrega.

## Domínio (DDD)

- `Note` é o aggregate. Os itens do todo fazem parte dele e são salvos juntos.
- `Kind` escolhe a `Rule` de cada tipo (Strategy):
  - sticky não tem título nem itens e aceita até 500 caracteres;
  - texto não tem itens;
  - todo não tem corpo e não aceita item vazio.
- `Notes` é o serviço de aplicação, com `add`, `edit`, `remove`, `push`, `mark` e `pull`. Cada escrita valida a nota, salva e publica um `Event`.
- `Blobs` é a porta dos bytes das imagens. A nota guarda só os dados de cada imagem (`Image`: id, tipo, tamanho, largura e altura), e `Notes::attach`/`detach` cuidam dos dois lados. `remove` apaga também as imagens da nota.
- `Repo` é a porta de persistência. `store::Sqlite` é a implementação, e os testes do domínio usam um repo em memória.
- `Bus` é o Observer. O servidor pendura nele o `Live`, que manda os eventos para o WebSocket. Um webhook, por exemplo, seria só mais um `Listener`.

## Ordem do grid

As fixadas vêm primeiro e as outras depois. Dentro de cada grupo, vale o `rank: f64`. Soltar um card entre A e B grava `(A + B) / 2` numa única nota, com um evento só. A nota nova entra acima de todas (`menor - 1`).

## Busca

- `domain::search` faz o `fold` (NFD sem marcas combinantes, em minúsculas) e o `hit`, que exige todos os termos no título, no corpo ou nos itens. A API usa isso em `?q=`.
- A paleta (Ctrl+K) faz a mesma busca no cliente, sobre a lista que já está em memória, e responde instantâneo. `web/src/lib/find/match.ts` mantém o tamanho do texto no `fold`, para o destaque cair no lugar certo do original. O ranking põe o que bate no título antes do que bate no corpo; empate fica com a mais recente.

## Banco

`store` migra pelo `pragma user_version`. Cada arquivo em `crates/store/src/sql/` é um passo, aplicado uma vez dentro de uma transação. Bancos antigos sobem de versão sozinhos ao abrir.

## Imagens

- No SQLite, os bytes ficam na tabela `blobs`, separada de `notes`. O backup continua sendo um arquivo só, e excluir uma nota leva as imagens dela junto.
- O cliente reduz antes de enviar (`web/src/lib/images/shrink.ts`): lado maior de no máximo 2048px, WebP a 85% (ou JPEG onde não há encoder WebP). GIF vai como está, e se a versão reduzida sair maior que o original, vai o original.
- Na interface, `Catch` captura colar e soltar, `Gallery` mostra o mosaico e `Viewer` abre em tela cheia (num portal no `<body>`, para escapar do contexto de empilhamento da view transition). No grid, a primeira imagem vira capa do card. No sticky, ela vira a foto de cima da polaroid.

## Sync

- Toda escrita volta como evento em `/api/live` para todos os clientes, inclusive quem escreveu.
- Conflito resolve por última escrita: o servidor carimba `updated`, e o cliente ignora evento mais velho que o que já tem.
- O cliente aplica a edição na hora (otimista) e manda um PATCH com debounce de 300 ms. Enquanto há edição pendente, ela fica por cima do que chega do servidor.
- Ao reconectar, o cliente recarrega a lista. Se ficou para trás no canal, o servidor manda `resync`.

## Interface

```
web/src/lib/
  api/      fetch + WebSocket (Observer)
  notes/    tipos, store reativo, aplicar evento, rank
  kinds/    um registro por tipo (Strategy): Card, Full, proporção, tamanho da janela, título
  grid/     masonry por seções (Fixadas / Outras) e drag dentro da seção
  find/     paleta Ctrl+K e busca
  images/   reduzir, colar/soltar, galeria e visualizador
  tabs/     abas locais do dispositivo, seletor do +, estado do sync
  shell/    destacar janela: Tauri (WebviewWindow) ou navegador (popup), escolhido em runtime
  ui/       ícones, movimento, tempo relativo
web/src/view/  Main (grid + abas), Open (nota aberta), Solo (janela destacada), Lock (token)
```

- O masonry põe cada nota na coluna mais baixa. Os cards são posicionados por `transform`, que anima sozinho ao reordenar.
- Abrir uma nota usa View Transitions: o card cresce até a área toda. Onde a API não existe, a nota só aparece.
- As abas ficam no `localStorage`, então cada dispositivo tem as suas.
