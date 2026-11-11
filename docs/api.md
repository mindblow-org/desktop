# API

Base: `http://localhost:7341` ou o link da tailnet. JSON em tudo.

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

`kind` é `sticky`, `text` ou `todo`. Cada tipo aceita uma forma; o que fugir dela volta `422` com `{"error": "..."}`:

- **sticky**: só `body`, até 500 caracteres;
- **text**: `title` e `body`;
- **todo**: `title` e `items`.

Todos os tipos aceitam imagens: texto e todo até 50, sticky uma só.

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
| GET | `/api/live` | WebSocket | eventos |
| GET | `/api/health` | | `{ok, version}` |

A ordem da lista põe as fixadas (`pinned: true`) primeiro e depois segue o `rank`.

`?q=` filtra no título, no corpo e nos itens, sem diferenciar maiúsculas nem acentos. Todos os termos precisam bater: `?q=mercado pão` só devolve notas que têm as duas palavras.

Imagens: `png`, `jpeg`, `webp` ou `gif`, até 20 MB. O servidor não decodifica a imagem, então `w` e `h` são de quem envia; sem eles, a interface só não reserva o espaço antes de carregar. SVG e qualquer outro tipo voltam `422`. Excluir a nota apaga as imagens dela.

Notas e itens aceitam o `id` vindo de quem cria. Se não vier, o servidor gera um UUID v7.

## Eventos (`/api/live`)

```json
{ "type": "created", "note": { … } }
{ "type": "updated", "note": { … } }
{ "type": "deleted", "id": "…" }
{ "type": "peers", "count": 3 }
{ "type": "resync" }
```

- `?d=<id do dispositivo>` faz várias janelas do mesmo aparelho contarem como um só em `peers`.
- `resync` avisa que o cliente perdeu eventos e precisa recarregar a lista.

## Acesso

- O servidor só escuta em `127.0.0.1`, e a tailnet chega até ele pelo `tailscale serve`.
- Só passam os Hosts `localhost`, `*.ts.net`, nomes curtos do MagicDNS e o que estiver em `MINDBLOW_HOSTS`. Requisição com `Origin` de outro site é recusada.
- Com `MINDBLOW_TOKEN` definido, toda `/api/*` exige `Authorization: Bearer <token>`. No WebSocket e no `<img>` de `/api/images`, o token vai em `?token=`.

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
