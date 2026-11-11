<p align="center">
  <img src="web/public/icon.svg" width="88" height="88" alt="">
</p>

<h1 align="center">mindblow</h1>

<p align="center">Notas minimalistas que moram em cada aparelho, funcionam sem internet e se sincronizam pela tailnet.</p>

<p align="center"><a href="https://mindblow-org.github.io/site/">Site</a> · <a href="https://github.com/mindblow-org/desktop/releases/latest">Baixar</a></p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/images/grid-dark.png">
    <img src="docs/images/grid-light.png" alt="Grid do mindblow com stickies, textos, listas de tarefas e um board">
  </picture>
</p>

Há quatro tipos de nota: **sticky**, **texto**, **todo** e **board**. Elas ficam num grid em que dá para arrastar os cards, abrem em abas e qualquer uma pode virar uma janela própria. Outros apps criam, mudam e tiram notas pela API.

Por baixo: servidor em Rust (axum + SQLite), interface em Svelte 5, app desktop em Tauri 2 e app Android. Os apps levam o servidor junto, com banco próprio; quando um aparelho alcança outro, eles trocam as mudanças campo a campo.

## Board

Um quadro branco simples: caneta, formas, setas, texto e imagens. Dá para selecionar vários, redimensionar, girar, copiar e colar (inclusive imagem com Ctrl+V).

<p align="center">
  <img src="docs/images/board.png" alt="Board com um diagrama do sync: celular e notebook ligados ao servidor dentro da tailnet, com a caixa do servidor selecionada">
</p>

## No celular

O [app Android](https://github.com/mindblow-org/android) guarda as notas no próprio celular e funciona sem internet. Para levar as notas a outro aparelho, o botão de QR no canto de cima mostra o código, liga o sync com outro mindblow (lendo o QR dele) e lista quem está conectado.

> [!WARNING]
> O app Android com o servidor junto ainda não foi testado num celular de verdade; falta validar com o aparelho plugado.

Sem o app, abra o link da tailnet no navegador e use "adicionar à tela inicial".

<p align="center">
  <img src="docs/images/mobile.png" alt="Três telas de celular: o grid de notas, o QR para conectar outro aparelho com a lista de dispositivos e um board no modo escuro">
</p>

## Instalar

Servidor e app desktop para Linux e Windows estão nas [releases](https://github.com/mindblow-org/desktop/releases/latest). Para compilar e instalar como serviço na sua máquina:

```sh
./deploy/install.sh      # compila, sobe o serviço e publica na tailnet; no fim imprime o link
cargo run -p mindblow-app --release   # janela desktop, com o servidor junto
```

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
