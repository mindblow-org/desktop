# API

Base: `http://localhost:7341` (o app desktop e o Android têm um servidor desses dentro) ou o link da tailnet. JSON em tudo.

## Nota

```json
{
  "id": "0193…",
  "kind": "todo",
  "title": "Mercado",
  "body": "",
  "items": [{ "id": "0193…", "text": "café", "done": false }],
  "images": [{ "id": "0193…", "mime": "image/webp", "size": 160812, "w": 2048, "h": 1536 }],
  "pinned": false,
  "rank": -3,
  "created": 1794413817178,
  "updated": 1794413817178
}
```

`kind` é `sticky`, `text`, `todo` ou `board`. Cada tipo aceita uma forma; o que fugir dela volta `422` com `{"error": "..."}`:

- **sticky**: só `body`, até 500 caracteres;
- **text**: `title` e `body`;
- **todo**: `title` e `items`;
- **board**: `title` e `body` com a cena em JSON, até 1 MB.

Imagens: texto, todo e board aceitam até 50, sticky uma só.

### Cena do board

O `body` de um board vazio é `""`. Com desenho, é um JSON assim:

```json
{
  "v": 1,
  "els": [
    { "id": "…", "t": "pen", "c": 0, "pts": [0, 0, 12, 8, 30, 10] },
    { "id": "…", "t": "rect", "c": 2, "x": 0, "y": 0, "w": 160, "h": 90, "r": 0.3 },
    { "id": "…", "t": "ellipse", "c": 3, "x": 200, "y": 0, "w": 120, "h": 90 },
    { "id": "…", "t": "arrow", "c": 1, "x1": 170, "y1": 45, "x2": 300, "y2": 45 },
    { "id": "…", "t": "text", "c": 0, "x": 20, "y": 30, "s": "fila de jobs", "f": 28 },
    { "id": "…", "t": "image", "c": 0, "img": "0193…", "x": 0, "y": 120, "w": 320, "h": 200 }
  ]
}
```

- `c` é a cor: 0 é a cor do texto do tema, 1 a 4 são vermelho, azul, verde e âmbar.
- `pts` é a lista plana `x, y, x, y…` do traço livre.
- `r` é a rotação em radianos, em volta do centro do elemento. Vale para todos menos a seta, que gira pelas pontas. Sem `r`, é 0.
- `f` é o tamanho da fonte do texto. Sem `f`, é 20.
- `img` é o id de uma imagem da própria nota (`images`), enviada pela rota de imagens.
- O servidor só confere que `els` é uma lista; o resto da forma é do cliente.
- A busca (`?q=`) lê o título e os textos (`t: "text"`) da cena, não o JSON.

## Rotas

| Método | Rota | Corpo | Resposta |
|---|---|---|---|
| GET | `/api/notes` | `?q=` opcional | lista, na ordem do grid |
| POST | `/api/notes` | `{kind, title?, body?, items?, pinned?}` | `201` nota |
| GET | `/api/notes/{id}` | | nota |
| PATCH | `/api/notes/{id}` | `{title?, body?, items?, pinned?, rank?}` | nota |
| DELETE | `/api/notes/{id}` | | `204` |
| POST | `/api/notes/{id}/items` | `{text, id?, done?}` | `201` nota |
| PATCH | `/api/notes/{id}/items/{item}` | `{text?, done?}` | nota |
| DELETE | `/api/notes/{id}/items/{item}` | | nota |
| POST | `/api/notes/{id}/images` | bytes da imagem, `Content-Type: image/*`, `?w=&h=` opcionais | `201` nota |
| DELETE | `/api/notes/{id}/images/{img}` | | nota |
| GET | `/api/images/{img}` | | bytes, com cache imutável |
| PUT | `/api/images/{img}` | bytes de uma imagem que veio de outro aparelho, `Content-Type: image/*` | `204` |
| GET | `/api/live` | WebSocket | eventos |
| GET | `/api/health` | | `{ok, version, link, node}` |
| GET | `/api/sync` | `?since=&limit=` | página do feed |
| POST | `/api/sync` | `{notes, gone}` | `{from, to, need}` |
| POST | `/api/sync/now` | | `204` |
| GET | `/api/links` | | estado do sync com cada link |
| POST | `/api/links` | `{url, token?}` | `201` lista |
| DELETE | `/api/links` | `?url=` | `204` |

A ordem da lista põe as fixadas (`pinned: true`) primeiro e depois segue o `rank`.

`?q=` filtra no título, no corpo e nos itens, sem diferenciar maiúsculas nem acentos. Todos os termos precisam bater: `?q=mercado pão` só devolve notas que têm as duas palavras.

Imagens: `png`, `jpeg`, `webp` ou `gif`, até 20 MB. O servidor não decodifica a imagem, então `w` e `h` são de quem envia; sem eles, a interface só não reserva o espaço antes de carregar. SVG e qualquer outro tipo voltam `422`. Excluir a nota apaga as imagens dela.

Notas e itens aceitam o `id` vindo de quem cria. Se não vier, o servidor gera um UUID v7.

## Eventos (`/api/live`)

```json
{ "type": "created", "note": { … } }
{ "type": "updated", "note": { … } }
{ "type": "deleted", "id": "…" }
{ "type": "peers", "count": 2, "list": [{ "id": "…", "name": "Notebook", "since": 1794413817178 }, { "id": "…", "name": "iPhone · Safari", "since": 1794413900000 }] }
{ "type": "resync" }
{ "type": "sync", "list": [{ "url": "https://pc.tail1234.ts.net:7341", "token": "…", "state": "ok", "at": 1794413817178 }] }
```

- `?d=<id do dispositivo>` faz várias janelas do mesmo aparelho contarem como um só em `peers`.
- `?n=<nome>` dá nome ao aparelho na lista (até 40 caracteres). Para renomear sem reconectar, o cliente manda `{"type":"name","name":"Celular"}` pelo próprio WebSocket, e todos recebem um `peers` novo.
- `list` vem em ordem de chegada; `since` é quando o aparelho conectou.
- `resync` avisa que o cliente perdeu eventos e precisa recarregar a lista.
- `list` de `peers` junta os aparelhos ligados a este com os que cada link do sync conta.
- `sync` traz o estado de cada link sempre que ele muda.

## Sync

Cada mindblow guarda um feed das próprias mudanças e sincroniza com os links que conhece. Quem usa a API normal não precisa de nada disto; é o que um mindblow fala com outro.

`GET /api/sync?since=12` (até 50 por página, ou `limit`):

```json
{
  "node": "0193…",
  "seq": 40,
  "more": false,
  "notes": [{ "note": { … }, "clock": { "title": 1794413817178, "body": 1794413817178, "pinned": 0, "rank": 0, "items": { "0193…": 1794413817178 } } }],
  "gone": [{ "id": "0193…", "at": 1794413900000 }]
}
```

- `seq` é a posição do feed até onde a página vai; a próxima começa dali. `node` é o id do aparelho, para um link que aponta para o próprio aparelho ser notado.
- `clock` diz quando cada campo, item e imagem mudou. Item ou imagem que está no `clock` e não na nota foi removido.
- `gone` são notas excluídas, com quando.

`POST /api/sync` leva `{notes, gone}` no mesmo formato e junta campo a campo com o que já existe (o carimbo mais novo ganha). Responde `{from, to, need}`: a posição do feed antes e depois do lote e as imagens que faltam, para quem mandou enviar por `PUT /api/images/{img}`.

`POST /api/sync/now` avisa que a rede do aparelho mudou: cada link reconecta na hora e sincroniza, sem esperar a próxima tentativa.

`POST /api/links` aceita o link como o QR traz (`https://pc.tail1234.ts.net:7341/#token=…`): fica a origem, e o `#token=` vira o token. Ligar de novo um link que já existe troca o token e mantém o que já foi sincronizado. O estado de cada link é `wait`, `ok`, `off` (não respondeu), `old` (mindblow sem sync), `locked` (token não serve), `self` (é este aparelho) ou `fail`, com `at` do último sync que deu certo.

## Acesso

- O servidor só escuta em `127.0.0.1`, e a tailnet chega até ele pelo `tailscale serve`.
- Só passam os Hosts `localhost`, `*.ts.net`, nomes curtos do MagicDNS e o que estiver em `MINDBLOW_HOSTS`. Requisição com `Origin` de outro site é recusada.
- Com `MINDBLOW_TOKEN` definido, toda `/api/*` exige `Authorization: Bearer <token>`. No WebSocket e no `<img>` de `/api/images`, o token vai em `?token=`.
- A interface aceita o token no fim do link, em `#token=<token>`: guarda no aparelho e tira do endereço. É o que o QR de "conectar outro aparelho" usa. O `#` não vai para o servidor nem para os logs, mas quem vir o QR na tela consegue entrar.
- `link` do `/api/health` é o link da tailnet (`MINDBLOW_LINK`), para o QR funcionar quando a página foi aberta por `localhost`.
- No app Android, o servidor de dentro do app exige um token aleatório, criado na instalação: `127.0.0.1` é o mesmo para todos os apps do celular.
- O token de cada link fica no banco do aparelho, e `GET /api/links` devolve ele para a interface montar o QR.

## Exemplos

```sh
curl -X POST localhost:7341/api/notes -H 'content-type: application/json' \
  -d '{"kind":"todo","title":"Deploy","items":[{"text":"rodar testes"}]}'

curl -X POST localhost:7341/api/notes/$ID/items -H 'content-type: application/json' \
  -d '{"text":"subir release"}'

curl -X POST "localhost:7341/api/notes/$ID/images?w=1200&h=750" \
  -H 'content-type: image/png' --data-binary @print.png

curl 'localhost:7341/api/notes?q=cafe'

curl -X PATCH localhost:7341/api/notes/$ID -H 'content-type: application/json' -d '{"pinned":true}'

websocat ws://localhost:7341/api/live
```
