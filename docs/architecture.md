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
  - todo não tem corpo e não aceita item vazio;
  - board não tem itens, e o corpo é a cena em JSON (`note::scene`).
- `Notes` é o serviço de aplicação, com `add`, `edit`, `remove`, `push`, `mark` e `pull`. Cada escrita valida a nota, salva e publica um `Event`.
- `Blobs` é a porta dos bytes das imagens. A nota guarda só os dados de cada imagem (`Image`: id, tipo, tamanho, largura e altura), e `Notes::attach`/`detach` cuidam dos dois lados. `remove` apaga também as imagens da nota.
- `Repo` é a porta de persistência. `store::Sqlite` é a implementação, e os testes do domínio usam um repo em memória.
- `Bus` é o Observer. O servidor pendura nele o `Live`, que manda os eventos para o WebSocket. Um webhook, por exemplo, seria só mais um `Listener`.

## Ordem do grid

As fixadas vêm primeiro e as outras depois. Dentro de cada grupo, vale o `rank: f64`. Soltar um card entre A e B grava `(A + B) / 2` numa única nota, com um evento só. A nota nova entra acima de todas (`menor - 1`).

## Busca

- `domain::search` faz o `fold` (NFD sem marcas combinantes, em minúsculas) e o `hit`, que exige todos os termos no título, no corpo ou nos itens. A API usa isso em `?q=`.
- A paleta (Ctrl+K) faz a mesma busca no cliente, sobre a lista que já está em memória, e responde instantâneo. `web/src/lib/find/match.ts` mantém o tamanho do texto no `fold`, para o destaque cair no lugar certo do original. O ranking põe o que bate no título antes do que bate no corpo; empate fica com a mais recente.

## Board

- A cena mora no `body` da nota, como JSON (formato em [API](api.md#cena-do-board)). Não tem tabela nem rota própria: sync, histórico do servidor e busca seguem o caminho das outras notas.
- `web/src/lib/kinds/board/scene.ts` tem as funções puras: ler e gravar a cena, caixa de cada elemento (girada ou não), acerto do clique (pela borda nas formas, pela caixa no texto e na imagem), seleção por retângulo (só o que cabe inteiro), simplificação do traço, mover, girar, escalar, redimensionar pelo canto e clonar.
- Rotação fica no elemento (`r`, em volta do centro da caixa sem giro). O clique desfaz o giro no ponto antes de testar, então a geometria de cada tipo continua simples. A seta não tem `r`: girar e escalar mexem direto nas pontas.
- Redimensionar um elemento prende o canto oposto, inclusive girado. Formas e traço esticam livre; texto e imagem mantêm a proporção (no texto cresce a fonte `f`). Um grupo escala por igual e gira em volta do centro comum.
- `Shapes` desenha a cena em SVG. O mesmo componente serve na miniatura do card (com `vector-effect: non-scaling-stroke`, para o traço não sumir quando a cena encolhe) e no quadro aberto. Seleção e alças são uma camada à parte, só no `Full`.
- `Full` guarda só no fim do gesto: arrastar, desenhar, girar e redimensionar mexem numa cópia local, e a nota recebe uma edição por gesto. O desfazer é local, uma pilha com os `body` anteriores.
- Imagens do board usam as imagens da nota (`Notes::attach`, tabela `blobs`); o elemento guarda só o id. Apagar a imagem da cena não apaga o arquivo na hora, para o desfazer funcionar. Ao fechar o board, a interface apaga as imagens que nenhum elemento usa mais.
- Copiar e colar usam os eventos `copy`/`paste` do navegador (funcionam sem HTTPS). O conteúdo vai como JSON `{"mindblow":"board","els":[…]}` em texto. Colando em outro board, as imagens são baixadas e enviadas de novo para a nota de destino.
- A câmera (posição e zoom) não é salva. Ao abrir, o quadro enquadra o desenho.
- Duas pessoas no mesmo board: vale a última escrita, com a cena inteira.

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

## Dispositivos

- `Live` guarda cada aparelho conectado (id, nome, desde quando) e conta janelas do mesmo aparelho como um só. Cada entrada, saída ou troca de nome manda `peers` com a contagem e a lista.
- O nome vem do cliente: o que a pessoa escreveu (no `localStorage`) ou um palpite pelo navegador (`web/src/lib/devices/name.ts`: "Linux · Chrome", "iPhone · Safari", "App desktop").
- "Conectar outro aparelho" (`Connect.svelte`) gera o QR no navegador com `uqr`. O link é o endereço da página, ou o `link` do `/api/health` quando a página foi aberta por `localhost`. Com token, ele vai no `#token=` do link.

## Interface

```
web/src/lib/
  api/      fetch + WebSocket (Observer)
  notes/    tipos, store reativo, aplicar evento, rank
  kinds/    um registro por tipo (Strategy): Card, Full, proporção, tamanho da janela, título, limite de imagens
  grid/     masonry por seções (Fixadas / Outras) e drag dentro da seção
  find/     paleta Ctrl+K e busca
  images/   reduzir, colar/soltar, galeria e visualizador
  tabs/     abas locais do dispositivo, seletor do +, estado do sync
  shell/    destacar janela e controles da janela (minimizar, maximizar, fechar): Tauri ou navegador, escolhido em runtime
  devices/  nome do aparelho e o popover de QR + dispositivos
  ui/       ícones, movimento, tempo relativo
web/src/view/  Main (grid + abas), Open (nota aberta), Solo (janela destacada), Lock (token)
```

- O masonry põe cada nota na coluna mais baixa. Os cards são posicionados por `transform`, que anima sozinho ao reordenar.
- Abrir uma nota usa View Transitions: o card cresce até a área toda. Onde a API não existe, a nota só aparece.
- As abas ficam no `localStorage`, então cada dispositivo tem as suas.
- No app Tauri as janelas não têm moldura do sistema. `Frame` põe minimizar, maximizar e fechar na barra de cima e no cabeçalho da janela destacada; as áreas vazias têm `data-tauri-drag-region`. No navegador, `Frame` não aparece.
