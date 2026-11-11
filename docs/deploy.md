# Deploy e configuração

## Servidor

```sh
./deploy/install.sh
```

O script faz quatro coisas:

1. Compila `web/` e o binário `mindblow`, com a interface embutida, e instala em `~/.local/bin`.
2. Cria e liga o serviço de usuário `mindblow.service`.
3. Publica na tailnet com `tailscale serve --bg --https=7341 http://127.0.0.1:7341`.
4. Imprime o link, por exemplo `https://maquina.tail1234.ts.net:7341`.

Ele pode rodar de novo sem problema, e é assim que se atualiza. Para outra porta: `MINDBLOW_PORT=7400 ./deploy/install.sh`. O script recusa porta que outro programa já ocupa.

Antes, ligue no Tailscale **MagicDNS** e **HTTPS Certificates**. Se o `tailscale serve` negar acesso, rode `sudo tailscale set --operator="$USER"` e tente de novo.

## Variáveis

| Variável | Padrão | |
|---|---|---|
| `MINDBLOW_PORT` | `7341` | porta local |
| `MINDBLOW_HOST` | `127.0.0.1` | interface de rede |
| `MINDBLOW_DB` | `~/.local/share/mindblow/mindblow.db` | banco SQLite |
| `MINDBLOW_TOKEN` | vazio | exige token na API |
| `MINDBLOW_HOSTS` | vazio | Hosts extras aceitos, separados por vírgula |
| `MINDBLOW_LINK` | link da tailnet, gravado pelo `install.sh` | link que o QR de "conectar outro aparelho" mostra |
| `MINDBLOW_PEER` | vazio | outro mindblow para sincronizar desde a partida (o link do QR, com ou sem `#token=`); mais links se ligam pela interface |

No serviço: `systemctl --user edit mindblow` e uma linha `Environment=MINDBLOW_TOKEN=...` em `[Service]`.

Logs: `journalctl --user -u mindblow -f`.

## App desktop

O app leva o servidor junto: ao abrir, ele sobe o mindblow em `localhost:7341` com o banco desta máquina (`~/.local/share/mindblow/mindblow.db`, o mesmo do serviço) e funciona sem rede. Se a porta já está ocupada, o serviço roda nesta máquina e a janela usa ele.

Para sincronizar com o mindblow de outra máquina, ligue o link dela no QR do canto de cima, ou defina `MINDBLOW_URL` ou escreva a URL em `~/.config/mindblow/url` (com `#token=` se ela pede token). Antes, esse endereço era o que a janela abria; agora ele vira um link do sync.

A janela não tem moldura do sistema: minimizar, maximizar e fechar ficam no canto direito da barra do app, e a área vazia da barra arrasta a janela (dois cliques maximizam). Em gerenciadores lado a lado, como o Hyprland, minimizar pode não fazer nada e arrastar só vale com a janela flutuando.

```sh
cargo run -p mindblow-app --release
(cd web && pnpm build) && cd app && pnpm dlx @tauri-apps/cli@2 build   # AppImage e .deb em target/release/bundle
```

## Release

Cada push na branch `release` dispara o workflow [release](../.github/workflows/release.yml), que publica a release `vX.Y.Z` no GitHub com:

- o servidor, com a interface embutida: `mindblow-linux-x64`, `mindblow-linux-arm64` e `mindblow-windows-x64.exe`;
- o app desktop, com o servidor junto: `.AppImage` e `.deb` no Linux, `-setup.exe` no Windows.

A versão vem de `[workspace.package]` no `Cargo.toml` (o app usa a mesma). Para soltar uma versão nova:

```sh
# suba a versão no Cargo.toml e commite na main
git push origin main:release
```

O workflow cria a release como rascunho, compila tudo e só publica no fim, então uma falha no meio não deixa release incompleta à vista. Se a versão já foi publicada, ele para e avisa para subir a versão. Os instaladores não são assinados: no Windows, o SmartScreen pede confirmação na primeira vez.

## Celular e outros PCs

O [app Android](https://github.com/mindblow-org/android) leva o servidor junto (o crate `crates/mobile`, compilado com `cargo ndk`) e guarda as notas no celular. Para sincronizar com o PC, toque no QR do canto de cima, em "ler QR", e leia o QR do mindblow do PC.

No navegador, abra o link da tailnet. No celular, "adicionar à tela inicial" instala como PWA, que depende do PC estar no ar. Para destacar uma nota em janela no navegador, permita popups.
