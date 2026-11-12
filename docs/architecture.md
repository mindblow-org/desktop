# Arquitetura

```
crates/domain   regras e casos de uso, sem IO
crates/store    Repo em SQLite
crates/server   HTTP, WebSocket, sync e a interface embutida (bin `mindblow`)
crates/mobile   o servidor como libmindblow.so, para o app Android
app/            janela Tauri, que leva o servidor junto
web/            interface Svelte 5
deploy/         systemd + tailscale serve
```

Cada aparelho com app roda o próprio mindblow, com banco próprio, e funciona sem rede: o app desktop sobe o servidor em `localhost:7341` (ou usa o serviço, se ele já ocupa a porta), e o app Android leva o mesmo servidor compilado como `libmindblow.so`. A interface é sempre a mesma, entregue pelo servidor do aparelho. O navegador/PWA é cliente de um mindblow da tailnet, como antes.

Quando um aparelho alcança outro, eles sincronizam (ver [Sync](#sync)). O jeito comum é o PC com o serviço fazer o papel de hub: fica ligado, tem link da tailnet, e o celular e o notebook sincronizam com ele. O protocolo é igual em todos, então qualquer mindblow pode ser link de qualquer outro.

## Domínio (DDD)

- `Note` é o aggregate. Os itens do todo fazem parte dele e são salvos juntos.
- `Kind` escolhe a `Rule` de cada tipo (Strategy):
  - sticky não tem título nem itens, aceita até 500 caracteres e é o único com cor (`Color`: cinza ou um de seis pastéis);
  - texto não tem itens;
  - todo não tem corpo e não aceita item vazio;
  - board não tem itens, e o corpo é a cena em JSON (`note::scene`).
- `Notes` é o serviço de aplicação, com `add`, `edit`, `remove`, `push`, `mark` e `pull`. Cada escrita valida a nota, carimba o que mudou (`Clock`), salva e publica um `Event`. Para o sync, `feed`, `take`, `adopt` e `lack`.
- `Blobs` é a porta dos bytes das imagens. A nota guarda só os dados de cada imagem (`Image`: id, tipo, tamanho, largura e altura), e `Notes::attach`/`detach` cuidam dos dois lados. `remove` apaga também as imagens da nota.
- `Repo` é a porta de persistência, com o feed de mudanças e as exclusões lembradas. `Links` guarda os outros aparelhos e o id deste. `store::Sqlite` implementa as duas, e os testes do domínio usam um repo em memória (`tests/mem`).
- `Bus` é o Observer. O servidor pendura nele o `Live`, que manda os eventos para o WebSocket, e o `Syncer`, que acorda o sync. Um webhook, por exemplo, seria só mais um `Listener`.

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
- Excluir a seleção: tecla Delete/Backspace ou a lixeira da barra, que fica apagada sem seleção.
- Copiar e colar usam os eventos `copy`/`paste` do navegador (funcionam sem HTTPS). O conteúdo vai como JSON `{"mindblow":"board","els":[…]}` em texto. Colando em outro board, as imagens são baixadas e enviadas de novo para a nota de destino.
- A câmera (posição e zoom) não é salva. Ao abrir, o quadro enquadra o desenho.
- Duas pessoas no mesmo board: vale a última escrita, com a cena inteira.

### Escanear caderno

- "Escanear caderno", no seletor do +, abre a câmera (`<input capture>`). No app Android, quem atende é o scanner de documento do Google Play Services, que já entrega a folha recortada e endireitada; no navegador vem a foto crua.
- `web/src/lib/scan/ink.ts` transforma a foto em cena, sem servidor e sem modelo: recorta na folha (a maior área clara e sem cor, para tirar mesa e teclado), separa a tinta com limiar adaptativo, tira pauta e margem (linhas claras, finas e longas), afina a tinta até 1 pixel (Zhang-Suen), rastreia o esqueleto em traços e simplifica (Douglas-Peucker). Contorno fechado grande com cara de caixa ou de círculo vira `rect` ou `ellipse`, girado se for o caso; a cor da caneta vira a cor do board, descontada a cor do papel.
- Letra continua letra de mão: vira traço, não texto, e não entra na busca.
- Roda num worker (`scan/worker.ts`). A foto é reduzida para 1400px no lado maior; a folha ocupa 900 de largura no board e as folhas ficam uma embaixo da outra. Se a cena passar de 1 MB, o traço é simplificado mais.
- O board nasce com a cena num `POST` só. A foto não é enviada.

## Banco

`store` migra pelo `pragma user_version`. Cada arquivo em `crates/store/src/sql/` é um passo, aplicado uma vez dentro de uma transação. Bancos antigos sobem de versão sozinhos ao abrir; no passo 4, as notas que já existiam entram no feed na ordem em que foram criadas.

## Imagens

- No SQLite, os bytes ficam na tabela `blobs`, separada de `notes`. O backup continua sendo um arquivo só, e excluir uma nota leva as imagens dela junto.
- O cliente reduz antes de enviar (`web/src/lib/images/shrink.ts`): lado maior de no máximo 2048px, WebP a 85% (ou JPEG onde não há encoder WebP). GIF vai como está, e se a versão reduzida sair maior que o original, vai o original.
- Na interface, `Catch` captura colar e soltar, `Gallery` mostra o mosaico e `Viewer` abre em tela cheia (num portal no `<body>`, para escapar do contexto de empilhamento da view transition). No grid, a primeira imagem vira capa do card. No sticky, ela vira a foto de cima da polaroid.

## Sync

### Na interface

- O cliente fala só com o servidor do aparelho. Toda escrita volta como evento em `/api/live` para todos os clientes dele, inclusive quem escreveu, e o que chega pelo sync também.
- O cliente aplica a edição na hora (otimista) e manda um PATCH com debounce de 300 ms. Enquanto há edição pendente, ela fica por cima do que chega do servidor, e evento mais velho que o que já tem é ignorado.
- Ao reconectar, o cliente recarrega a lista. Se ficou para trás no canal, o servidor manda `resync`.

### Entre aparelhos

- **Feed.** Cada escrita no banco pega a próxima posição (`meta.seq`) e grava em `notes.seq`, ou em `gone.seq` quando é uma exclusão. `GET /api/sync?since=` devolve o que mudou depois de uma posição, 50 por página, com o id do aparelho.
- **Relógio por campo.** Cada nota guarda em `clock` quando título, corpo, fixar, cor, ordem, cada item e cada imagem mudaram. Item ou imagem que está no relógio e não na nota foi removido; isso fica lembrado por 90 dias. O PATCH com a lista inteira de itens é comparado item a item, então só o que mudou ganha carimbo novo.
- **Junção.** `merge` fica, em cada campo, com o carimbo mais novo; itens e imagens são juntados um a um, e a ordem é a da versão mais nova. O resultado é o mesmo nos dois sentidos, então os aparelhos convergem. Se a junção quebrar a regra do tipo (51 imagens, por exemplo), vale a versão mais nova inteira.
- **Relógio híbrido.** O carimbo é o maior entre o relógio da máquina e o último visto + 1. Uma edição feita depois de receber outra ganha dela, mesmo com o relógio do celular atrasado. Carimbo mais de um dia no futuro não puxa o relógio.
- **Exclusão.** Excluir guarda a nota em `gone`. Ela some do outro lado se lá ela não foi editada depois; se foi, a edição ganha e a nota volta para quem excluiu. Exclusão de nota que este aparelho nem tem também fica guardada, para uma cópia velha vinda de um terceiro não ressuscitar a nota.
- **Syncer.** Para cada link (tabela `links`), um laço puxa o feed de lá (cursor `pull`), junta com `take` e depois empurra o daqui (cursor `push`). `take` só grava o que muda algo, então o que volta para onde saiu não ecoa. Ele também devolve a posição do feed antes e depois (`Took`): se nada daqui estava esperando, o que entrou não precisa voltar, e o cursor anda junto. O mesmo vale do outro lado no `POST /api/sync`.
- **Imagens.** A nota anda sem os bytes. Quem recebe responde as imagens que faltam (`need`), e elas vão por `PUT /api/images/{id}` ou são baixadas por `GET`. Na primeira volta de cada link, `lack` procura as que ficaram para trás.
- **Quando sincroniza.** Mudança daqui (pelo `Bus`), mudança de lá (o laço fica ligado no `/api/live` do link) ou um tempo sem nada: cinco minutos com o `/api/live` de lá ligado, que é por onde os avisos chegam, e um minuto sem ele. Depois de um aviso, o laço espera 2 s juntando outros, então digitar ou desenhar num board manda uma remessa a cada 2 s, não uma por palavra ou por traço. Sem resposta, a espera dobra de 1 s até 30 s, e o `/api/live` de lá voltar acorda o laço na hora. Quando a rede do aparelho muda, `POST /api/sync/now` corta a espera: o `/api/live` de cada link reconecta na hora (a conexão antiga pode ter morrido com a rede velha sem ninguém saber), e reconectar acorda o laço. O app Android chama ele quando a rede volta. Token que não serve ou link que é este próprio aparelho param o laço até o link mudar.
- **Estado.** Cada link está `wait`, `ok`, `off`, `old` (mindblow sem sync), `locked`, `self` ou `fail`. Ele vem em `GET /api/links` e no evento `sync`.
- **Dados.** O feed e as notas vão em gzip (um board é JSON e encolhe uns 3x): as respostas, pelo `Accept-Encoding`, e o lote do `POST /api/sync`, quando a página do feed de lá diz `gzip: true`; com mindblow antigo, o lote vai cru. No `/api/live`, o laço entra com `lean=true` e recebe os eventos de nota sem a nota (só tipo e id), porque ela vem pelo feed. Num board de 271 KB, cada traço feito no PC custava 593 KB no 4G do celular (a nota vinha pelo `/api/live` e de novo pelo feed, crua); agora custa uns 100 KB.
- **TLS.** O cliente usa rustls com ring e as raízes do `webpki-roots`, sem o verificador do sistema, para compilar igual no Linux, no Windows e no Android.
- **Limites.** O mesmo campo editado nos dois lados fica com a edição mais nova: dois textos no mesmo corpo não se misturam, e um board é um campo só (a cena inteira). No Android, com o app fechado, quem sobe o que ficou só no celular é um job do sistema que roda quando há rede (ver o README do [app Android](https://github.com/mindblow-org/android)); o que muda no PC só desce quando o app abre.

## Dispositivos

- `Live` guarda cada aparelho conectado (id, nome, desde quando) e conta janelas do mesmo aparelho como um só. Cada entrada, saída ou troca de nome manda `peers` com a contagem e a lista.
- O laço de cada link entra no `/api/live` de lá com o id e o nome deste aparelho (o da janela mais antiga que tem nome), e a lista que vem de lá entra na daqui. Assim o celular aparece no PC, e o PC e o notebook aparecem no celular.
- O nome vem do cliente: o que a pessoa escreveu (no `localStorage`) ou um palpite pelo navegador (`web/src/lib/devices/name.ts`: "Linux · Chrome", "iPhone · Safari", "App desktop", "App Android").
- "Conectar outro aparelho" (`Connect.svelte`) gera o QR no navegador com `uqr`. O link é o endereço da página; aberto por `localhost` (o app), é o `link` do `/api/health`, e sem ele, o do aparelho com que este sincroniza. Com token, ele vai no `#token=` do link.
- O mesmo popover lista os links do sync, liga um novo (colando o link do QR de outro mindblow) e para de sincronizar. No app Android, "ler QR" abre a câmera pelo `window.mindblowAndroid.scan()`, e o link volta em `window.mindblowLink(url)`.

## Interface

```
web/src/lib/
  api/      fetch + WebSocket (Observer)
  notes/    tipos, store reativo, aplicar evento, rank
  kinds/    um registro por tipo (Strategy): Card, Full, proporção, tamanho da janela, título, limite de imagens
  grid/     masonry por seções (Fixadas / Outras) e drag dentro da seção
  find/     paleta Ctrl+K e busca
  images/   reduzir, colar/soltar, galeria e visualizador
  scan/     foto de folha de caderno vira cena do board
  tabs/     abas locais do dispositivo, seletor do +, estado do sync
  shell/    destacar janela e controles da janela (minimizar, maximizar, fechar): Tauri ou navegador, escolhido em runtime
  devices/  nome do aparelho e o popover de QR + dispositivos
  ui/       ícones, movimento, tempo relativo
web/src/view/  Main (grid + abas), Open (nota aberta), Solo (janela destacada), Lock (token)
```

- O masonry põe cada nota na coluna mais baixa. Os cards são posicionados por `transform`, que anima sozinho ao reordenar.
- Abrir uma nota usa View Transitions: o card cresce até a área toda. Onde a API não existe, a nota só aparece.
- As abas ficam no `localStorage`, então cada dispositivo tem as suas.
- No celular (até 640px) não há abas: abrir uma nota troca a que está aberta, e voltar ao grid fecha ela (`tabs.solo`). A volta é pelo `< Notas` da nota ou pelo botão voltar do app Android.
- No celular também não há barra em cima: o `Bar` vira uma toolbar flutuante embaixo, no centro (QR, nova nota, busca), e some com uma nota aberta. Aí aparece a da nota (`Open`: imagem, fixar, excluir). No board a toolbar troca a imagem pelo botão de ferramentas, que liga e desliga a barra do board logo acima (`kit`); o voltar fecha a barra antes de sair do board. `--dock` é o espaço que a toolbar ocupa embaixo, e o grid, a nota e as folhas do `Picker` e do `Connect` respeitam ele.
- A cor do sticky é o papel: `data-color` no card do grid e no sticky aberto troca `--sticky` por um dos pastéis (no escuro, versões apagadas), e o texto segue na cor de sempre. `kinds/sticky/Colors` são as bolinhas na linha de baixo do sticky aberto. O desenho está no Mocha, blend v4 do mindblow.
- Nota que sai vazia some (`Spec.empty`), mas imagem conta como conteúdo: a sticky só com foto, que o compartilhar do Android cria, fica.
- No app Tauri as janelas não têm moldura do sistema. `Frame` põe minimizar, maximizar e fechar na barra de cima e no cabeçalho da janela destacada; as áreas vazias têm `data-tauri-drag-region`. No navegador, `Frame` não aparece.
