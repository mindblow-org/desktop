#!/usr/bin/env bash
set -euo pipefail

REPO="mindblow-org/desktop"
PORT="${MINDBLOW_PORT:-7341}"
BIN="$HOME/.local/bin"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS="$DATA/applications"
ICONS="$DATA/icons/hicolor"
UNIT="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/mindblow.service"

say() { printf '\n\033[1m==> %s\033[0m\n' "$1"; }
warn() { printf '!!  %s\n' "$1" >&2; }
die() { printf 'x   %s\n' "$1" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

usage() {
  cat <<'EOF'
mindblow para Linux

  curl -fsSL https://github.com/mindblow-org/desktop/releases/latest/download/install.sh | bash
  curl -fsSL https://github.com/mindblow-org/desktop/releases/latest/download/install.sh | bash -s -- --servidor

sem opção     app desktop, com ícone e entrada no menu
--servidor    serviço sempre ligado, publicado na tailnet (MINDBLOW_PORT muda a porta)
--fonte       o serviço compilado deste checkout: ./install.sh --fonte
--versao v…   uma versão em vez da mais nova
--remover     tira app e serviço; as notas ficam

Rodar de novo atualiza.
EOF
}

fuse() {
  [ -e /dev/fuse ] && { have fusermount3 || have fusermount; }
}

server=0 remove=0 source=0 tag=""
while [ $# -gt 0 ]; do
  case "$1" in
    --servidor) server=1 ;;
    --remover) remove=1 ;;
    --fonte) source=1 server=1 ;;
    --versao) tag="${2:?--versao pede a versão, como v0.3.0}"; shift ;;
    -h | --help) usage; exit 0 ;;
    *) die "opção desconhecida: $1" ;;
  esac
  shift
done

[ "$(uname -s)" = Linux ] || die "Este instalador é para Linux. No Windows, use o -setup.exe da release."
case "$(uname -m)" in
  x86_64 | amd64) arch=x64 ;;
  aarch64 | arm64) arch=arm64 ;;
  *) die "Arquitetura $(uname -m) sem build pronto." ;;
esac

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

latest() {
  have curl || die "curl não encontrado."
  local u
  u="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest")" || die "Sem acesso ao GitHub."
  printf '%s' "${u##*/}"
}

fetch() {
  local bar=-s
  [ -t 2 ] && bar=--progress-bar
  curl -fSL "$bar" -o "$2" "https://github.com/$REPO/releases/download/$tag/$1" || die "Não deu para baixar $1 da $tag."
}

app() {
  [ "$arch" = x64 ] || die "O app desktop só tem build para x86_64. Neste aparelho, use --servidor."
  say "App desktop $tag"
  fetch "mindblow_${tag#v}_amd64.AppImage" "$TMP/mindblow.AppImage"
  chmod +x "$TMP/mindblow.AppImage"
  (cd "$TMP" && ./mindblow.AppImage --appimage-extract 'usr/share/*' >/dev/null)
  mkdir -p "$BIN" "$APPS"
  rm -rf "$DATA/mindblow/app" "$BIN/mindblow-app"
  if fuse; then
    install -m 755 "$TMP/mindblow.AppImage" "$BIN/mindblow-app"
  else
    warn "Sem FUSE: o app vai extraído, sem o AppImage."
    (cd "$TMP" && rm -rf squashfs-root && ./mindblow.AppImage --appimage-extract >/dev/null)
    mkdir -p "$DATA/mindblow"
    mv "$TMP/squashfs-root" "$DATA/mindblow/app"
    printf '#!/bin/sh\nexec "%s" "$@"\n' "$DATA/mindblow/app/AppRun" > "$BIN/mindblow-app"
    chmod 755 "$BIN/mindblow-app"
  fi
  local root="$TMP/squashfs-root/usr/share"
  [ -d "$root" ] || root="$DATA/mindblow/app/usr/share"
  for f in "$root"/icons/hicolor/*/apps/mindblow-app.png; do
    local size
    size="$(basename "$(dirname "$(dirname "$f")")")"
    mkdir -p "$ICONS/$size/apps"
    install -m 644 "$f" "$ICONS/$size/apps/mindblow-app.png"
  done
  sed -e "s|^Exec=.*|Exec=$BIN/mindblow-app|" -e "/^TryExec=/d" "$root/applications/mindblow.desktop" > "$APPS/mindblow.desktop"
  printf 'TryExec=%s\n' "$BIN/mindblow-app" >> "$APPS/mindblow.desktop"
  have update-desktop-database && update-desktop-database "$APPS" >/dev/null 2>&1 || true
  have gtk-update-icon-cache && gtk-update-icon-cache -q -t "$ICONS" >/dev/null 2>&1 || true
  printf '    no menu como "mindblow", ou pelo terminal: mindblow-app\n'
}

serve() {
  have systemctl || die "systemd não encontrado."
  busy="$(ss -ltnpH "sport = :$PORT and ( src 127.0.0.1 or src 0.0.0.0 or src [::1] or src [::] )" 2>/dev/null | grep -v '"mindblow"' || true)"
  [ -z "$busy" ] || die "A porta $PORT já está em uso por outro programa. Escolha outra: MINDBLOW_PORT=<porta>"
  mkdir -p "$BIN"
  if [ "$source" = 1 ]; then
    local dir
    dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    [ -f "$dir/Cargo.toml" ] || die "--fonte roda de dentro do repo: ./install.sh --fonte"
    have cargo || die "Rust (cargo) não encontrado."
    have pnpm || die "pnpm não encontrado: npm install -g pnpm"
    say "Compilando do código"
    (cd "$dir/web" && pnpm install --frozen-lockfile && pnpm build)
    (cd "$dir" && cargo build --release -p server)
    install -m 755 "$dir/target/release/mindblow" "$BIN/mindblow"
  else
    say "Servidor $tag"
    fetch "mindblow-linux-$arch" "$TMP/mindblow"
    install -m 755 "$TMP/mindblow" "$BIN/mindblow"
  fi

  local url="http://localhost:$PORT" link=""
  if have tailscale; then
    say "Publicando na tailnet"
    if tailscale serve --bg --https="$PORT" "http://127.0.0.1:$PORT" >/dev/null 2>&1; then
      found="$(tailscale serve status 2>/dev/null | awk -v p=":$PORT" '/^https:\/\// && index($1, p) { print $1; exit }')"
      [ -n "$found" ] && url="$found" && link="$found"
    else
      warn "tailscale serve falhou. Se for permissão: sudo tailscale set --operator=\"$USER\""
    fi
  else
    warn "Tailscale não encontrado: só o acesso local vai funcionar."
  fi

  say "Serviço systemd"
  mkdir -p "$(dirname "$UNIT")"
  cat > "$UNIT" <<EOF
[Unit]
Description=mindblow (notas na tailnet)
After=network-online.target

[Service]
ExecStart=$BIN/mindblow
Environment=MINDBLOW_PORT=$PORT
Environment=MINDBLOW_LINK=$link
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
EOF
  systemctl --user daemon-reload
  systemctl --user enable mindblow >/dev/null
  systemctl --user restart mindblow
  printf '    %s\n' "$url"
}

drop() {
  say "Removendo"
  if [ -f "$UNIT" ]; then
    local p
    p="$(sed -n 's/^Environment=MINDBLOW_PORT=//p' "$UNIT")"
    PORT="${p:-$PORT}"
    systemctl --user disable --now mindblow >/dev/null 2>&1 || true
    rm -f "$UNIT"
    systemctl --user daemon-reload || true
    have tailscale && tailscale serve --https="$PORT" off >/dev/null 2>&1 || true
  fi
  rm -rf "$DATA/mindblow/app"
  rm -f "$BIN/mindblow" "$BIN/mindblow-app" "$APPS/mindblow.desktop" "$ICONS"/*/apps/mindblow-app.png
  have update-desktop-database && update-desktop-database "$APPS" >/dev/null 2>&1 || true
  printf '    as notas ficam em %s/mindblow/mindblow.db\n' "$DATA"
}

if [ "$remove" = 1 ]; then
  drop
  exit 0
fi

[ -n "$tag" ] || [ "$source" = 1 ] || tag="$(latest)"
if [ "$server" = 1 ]; then serve; else app; fi
say "Pronto"
