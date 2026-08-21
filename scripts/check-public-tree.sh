#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/check-public-tree.sh [--root PATH]

Scan tracked files for machine-specific rss-scout configuration and real
Notion identifiers. Placeholders and loopback URLs outside canonical TOML feed
configs are allowed.
USAGE
}

ROOT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      [[ $# -ge 2 ]] || { printf '%s\n' '--root requires a path' >&2; exit 2; }
      ROOT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'unknown option: %s\n' "$1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$ROOT" ]]; then
  ROOT="$(git rev-parse --show-toplevel)"
fi

git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 "$SCRIPT_DIR/check_public_tree.py" --root "$ROOT"
