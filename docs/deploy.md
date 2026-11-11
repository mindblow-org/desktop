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

No serviço: `systemctl --user edit mindblow` e uma linha `Environment=MINDBLOW_TOKEN=...` em `[Service]`.

Logs: `journalctl --user -u mindblow -f`.

## App desktop

O app é só uma janela que carrega o servidor. Por padrão ele abre `http://localhost:7341`. Para outra máquina, defina `MINDBLOW_URL` ou escreva a URL em `~/.config/mindblow/url`.

A janela não tem moldura do sistema: minimizar, maximizar e fechar ficam no canto direito da barra do app, e a área vazia da barra arrasta a janela (dois cliques maximizam). Em gerenciadores lado a lado, como o Hyprland, minimizar pode não fazer nada e arrastar só vale com a janela flutuando.

```sh
cargo run -p mindblow-app --release
cd app && pnpm dlx @tauri-apps/cli@2 build   # AppImage e .deb em target/release/bundle
```

## Celular e outros PCs

Abra o link da tailnet no navegador. No celular, "adicionar à tela inicial" instala como PWA. Para destacar uma nota em janela no navegador, permita popups.
