#!/usr/bin/env bash
# Generate a current-user launchd job for rss-scout.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: ./install-launchd.sh [options]

Options:
  --feeds PATH              feeds.toml to use in the scheduled job
                            (default: ./feeds.toml next to this script)
  --data-dir PATH           rss-scout data directory
                            (default: $RSS_SCOUT_DIR or ~/.rss-scout)
  --bin PATH                installed rss-scout binary
                            (default: $RSS_SCOUT_BIN or $CARGO_HOME/bin/rss-scout)
  --plist PATH              plist output path
                            (default: ~/Library/LaunchAgents/com.user.rss-scout.plist)
  --label LABEL             launchd label (default: com.user.rss-scout)
  --skip-cargo-install      do not run cargo install --path .
  --load                    load the generated LaunchAgent after writing it
  --dry-run                 print the generated plist to stdout
  -h, --help                show this help
USAGE
}

die() {
  printf 'install-launchd.sh: %s\n' "$*" >&2
  exit 1
}

require_value() {
  local flag="$1"
  local value="${2:-}"
  [[ -n "$value" ]] || die "$flag requires a value"
}

abs_path() {
  local path="$1"
  if [[ "$path" == /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s/%s\n' "$(pwd)" "$path"
  fi
}

xml_escape() {
  printf '%s' "$1" |
    sed \
      -e 's/&/\&amp;/g' \
      -e 's/</\&lt;/g' \
      -e 's/>/\&gt;/g' \
      -e 's/"/\&quot;/g' \
      -e "s/'/\&apos;/g"
}

write_plist() {
  local label_xml binary_xml feeds_xml data_dir_xml log_xml
  label_xml="$(xml_escape "$LABEL")"
  binary_xml="$(xml_escape "$BINARY_PATH")"
  feeds_xml="$(xml_escape "$FEEDS_PATH")"
  data_dir_xml="$(xml_escape "$DATA_DIR")"
  log_xml="$(xml_escape "$LOG_PATH")"

  cat <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${label_xml}</string>
  <key>ProgramArguments</key>
  <array>
    <string>${binary_xml}</string>
    <string>run</string>
    <string>--feeds</string>
    <string>${feeds_xml}</string>
    <string>--data-dir</string>
    <string>${data_dir_xml}</string>
  </array>
  <key>StartCalendarInterval</key>
  <array>
    <dict>
      <key>Hour</key>
      <integer>8</integer>
      <key>Minute</key>
      <integer>0</integer>
    </dict>
  </array>
  <key>StandardOutPath</key>
  <string>${log_xml}</string>
  <key>StandardErrorPath</key>
  <string>${log_xml}</string>
</dict>
</plist>
PLIST
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEEDS_PATH="${RSS_SCOUT_FEEDS:-$SCRIPT_DIR/feeds.toml}"
DATA_DIR="${RSS_SCOUT_DIR:-$HOME/.rss-scout}"
BINARY_PATH="${RSS_SCOUT_BIN:-${CARGO_HOME:-$HOME/.cargo}/bin/rss-scout}"
PLIST_PATH="${RSS_SCOUT_PLIST:-$HOME/Library/LaunchAgents/com.user.rss-scout.plist}"
LABEL="${RSS_SCOUT_LABEL:-com.user.rss-scout}"
RUN_CARGO_INSTALL=true
LOAD_AGENT=false
DRY_RUN=false

if [[ "${RSS_SCOUT_SKIP_CARGO_INSTALL:-}" == "1" ]]; then
  RUN_CARGO_INSTALL=false
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --feeds)
      require_value "$1" "${2:-}"
      FEEDS_PATH="$2"
      shift 2
      ;;
    --data-dir)
      require_value "$1" "${2:-}"
      DATA_DIR="$2"
      shift 2
      ;;
    --bin)
      require_value "$1" "${2:-}"
      BINARY_PATH="$2"
      shift 2
      ;;
    --plist)
      require_value "$1" "${2:-}"
      PLIST_PATH="$2"
      shift 2
      ;;
    --label)
      require_value "$1" "${2:-}"
      LABEL="$2"
      shift 2
      ;;
    --skip-cargo-install)
      RUN_CARGO_INSTALL=false
      shift
      ;;
    --load)
      LOAD_AGENT=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      RUN_CARGO_INSTALL=false
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

FEEDS_PATH="$(abs_path "$FEEDS_PATH")"
DATA_DIR="$(abs_path "$DATA_DIR")"
BINARY_PATH="$(abs_path "$BINARY_PATH")"
PLIST_PATH="$(abs_path "$PLIST_PATH")"
LOG_PATH="$DATA_DIR/scout.log"

[[ -f "$FEEDS_PATH" ]] || die "feeds config not found: $FEEDS_PATH"

if "$DRY_RUN"; then
  write_plist
  exit 0
fi

if "$RUN_CARGO_INSTALL"; then
  cargo install --path "$SCRIPT_DIR"
fi

mkdir -p "$DATA_DIR" "$(dirname "$PLIST_PATH")"

tmp_plist="$(mktemp "${PLIST_PATH}.XXXXXX")"
write_plist > "$tmp_plist"

if command -v plutil >/dev/null 2>&1; then
  plutil -lint "$tmp_plist" >/dev/null
fi

mv "$tmp_plist" "$PLIST_PATH"

printf 'Installed launchd plist: %s\n' "$PLIST_PATH"
printf 'rss-scout binary: %s\n' "$BINARY_PATH"
printf 'feeds config: %s\n' "$FEEDS_PATH"
printf 'data dir: %s\n' "$DATA_DIR"

if "$LOAD_AGENT"; then
  if ! launchctl unload "$PLIST_PATH" >/dev/null 2>&1; then
    printf 'No existing LaunchAgent was loaded for: %s\n' "$LABEL"
  fi
  launchctl load "$PLIST_PATH"
  printf 'Loaded LaunchAgent: %s\n' "$LABEL"
else
  printf 'Load it with: launchctl load %s\n' "$PLIST_PATH"
fi
