#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${MINDBLOW_PORT:-7341}"
BIN="$HOME/.local/bin"
UNIT="$HOME/.config/systemd/user/mindblow.service"

say() { printf '\n\033[1m==> %s\033[0m\n' "$1"; }
warn() { printf '!!  %s\n' "$1" >&2; }
die() { printf 'x   %s\n' "$1" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

have cargo || die "Rust (cargo) não encontrado."
have pnpm || die "pnpm não encontrado: npm install -g pnpm"
have systemctl || die "systemd não encontrado."

busy="$(ss -ltnpH "sport = :$PORT" 2>/dev/null | grep -v '"mindblow"' || true)"
[ -z "$busy" ] || die "A porta $PORT já está em uso por outro programa. Escolha outra: MINDBLOW_PORT=<porta> ./deploy/install.sh"

say "Compilando a interface"
(cd "$DIR/web" && pnpm install --frozen-lockfile && pnpm build)

say "Compilando o servidor"
(cd "$DIR" && cargo build --release -p server)
mkdir -p "$BIN"
install -m 755 "$DIR/target/release/mindblow" "$BIN/mindblow"

URL="http://localhost:$PORT"
LINK=""
if have tailscale; then
  say "Publicando na tailnet"
  if tailscale serve --bg --https="$PORT" "http://127.0.0.1:$PORT" >/dev/null 2>&1; then
    found="$(tailscale serve status 2>/dev/null | awk -v p=":$PORT" '/^https:\/\// && index($1, p) { print $1; exit }')"
    [ -n "$found" ] && URL="$found" && LINK="$found"
  else
    warn "tailscale serve falhou. Se for permissão: sudo tailscale set --operator=\"$USER\""
  fi
else
  warn "Tailscale não encontrado: só o acesso local vai funcionar."
fi

say "Serviço systemd"
mkdir -p "$(dirname "$UNIT")"
sed -e "s|@BIN@|$BIN/mindblow|" -e "s|@PORT@|$PORT|" -e "s|@LINK@|$LINK|" "$DIR/deploy/mindblow.service" > "$UNIT"
systemctl --user daemon-reload
systemctl --user enable mindblow >/dev/null
systemctl --user restart mindblow

say "Pronto"
printf '    %s\n' "$URL"
