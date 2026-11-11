# mindblow

Notas minimalistas que moram numa máquina só e aparecem em todos os seus dispositivos pela tailnet.

Há quatro tipos de nota: **sticky**, **texto**, **todo** e **board** (um quadro branco simples). Elas ficam num grid em que dá para arrastar os cards, abrem em abas e qualquer uma pode virar uma janela própria. Outros apps criam, mudam e tiram notas pela API.

Por baixo: servidor em Rust (axum + SQLite), interface em Svelte 5 e app desktop em Tauri 2.

## Instalar

```sh
./deploy/install.sh      # compila, sobe o serviço e publica na tailnet; no fim imprime o link
cargo run -p mindblow-app --release   # janela desktop, abre o servidor local
```

No celular, abra o link e use "adicionar à tela inicial".

## Desenvolver

```sh
cargo run -p server      # API em :7341
cd web && pnpm dev       # interface em :5341, com proxy para a API
cargo test && (cd web && pnpm test && pnpm check)
```

## Docs

- [Arquitetura](docs/architecture.md)
- [API](docs/api.md)
- [Deploy e configuração](docs/deploy.md)

## Licença

[MIT](LICENSE)
