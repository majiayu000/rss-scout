#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCANNER="$REPO_ROOT/scripts/check-public-tree.sh"
FIXTURE_ROOT="$(mktemp -d)"
trap 'rm -rf "$FIXTURE_ROOT"' EXIT

git -C "$FIXTURE_ROOT" init -q
git -C "$FIXTURE_ROOT" config user.name fixture
git -C "$FIXTURE_ROOT" config user.email fixture@example.invalid

cat > "$FIXTURE_ROOT/feeds.toml" <<'TOML'
[settings]
keywords = "example"

[[feeds]]
name = "Example"
url = "https://example.com/feed.xml"
TOML
cat > "$FIXTURE_ROOT/README.md" <<'MARKDOWN'
Safe placeholders are allowed:

```toml
database_id = "<your-notion-database-id>"
```

A loopback health-check example outside a canonical feed config is also safe:
http://127.0.0.1:8080/health

Template paths are safe: /Users/<user>/.rss-scout
MARKDOWN
git -C "$FIXTURE_ROOT" add .
"$SCANNER" --root "$FIXTURE_ROOT" >/dev/null

expect_failure() {
  local label="$1"
  local secret="${2:-}"
  local output=""
  if output="$("$SCANNER" --root "$FIXTURE_ROOT" 2>&1)"; then
    printf 'expected scanner failure for: %s\n' "$label" >&2
    exit 1
  fi
  if [[ -n "$secret" && "$output" == *"$secret"* ]]; then
    printf 'scanner leaked matched content for: %s\n' "$label" >&2
    exit 1
  fi
}

private_home='/''Users/private-user/research'
printf '%s\n' "$private_home" > "$FIXTURE_ROOT/private-path.txt"
git -C "$FIXTURE_ROOT" add private-path.txt
expect_failure 'private user path'
git -C "$FIXTURE_ROOT" rm -q -f private-path.txt

binary_home='/''Users/binary-user/research'
printf '\0%s\0' "$binary_home" > "$FIXTURE_ROOT/private-path.bin"
git -C "$FIXTURE_ROOT" add private-path.bin
expect_failure 'private user path in binary file' "$binary_home"
git -C "$FIXTURE_ROOT" rm -q -f private-path.bin

symlink_target="$FIXTURE_ROOT/"'Us''ers/private-user/research'
mkdir -p "$(dirname "$symlink_target")"
printf '%s\n' 'safe fixture content' > "$symlink_target"
ln -s "$symlink_target" "$FIXTURE_ROOT/private-path-link"
git -C "$FIXTURE_ROOT" add private-path-link
expect_failure 'private user path in symlink target' "$symlink_target"
git -C "$FIXTURE_ROOT" rm -q -f private-path-link

private_label='com.li''fcc.rss-scout'
printf '%s\n' "$private_label" > "$FIXTURE_ROOT/private-label.txt"
git -C "$FIXTURE_ROOT" add private-label.txt
expect_failure 'private launchd label'
git -C "$FIXTURE_ROOT" rm -q -f private-label.txt

private_repo='~/Desktop/code/AI/''tools/rss-scout'
printf '%s\n' "$private_repo" > "$FIXTURE_ROOT/private-repo.txt"
git -C "$FIXTURE_ROOT" add private-repo.txt
expect_failure 'machine-specific repository path'
git -C "$FIXTURE_ROOT" rm -q -f private-repo.txt

notion_id='11111111-2222-3333-4444-''555555555555'
printf 'database_id = "%s"\n' "$notion_id" > "$FIXTURE_ROOT/private-notion.toml"
git -C "$FIXTURE_ROOT" add private-notion.toml
expect_failure 'real Notion identifier'
git -C "$FIXTURE_ROOT" rm -q -f private-notion.toml

data_source_id='aaaaaaaa-bbbb-cccc-dddd-''eeeeeeeeeeee'
printf '{"data_source_id": "%s"}\n' "$data_source_id" > "$FIXTURE_ROOT/private-data-source.json"
git -C "$FIXTURE_ROOT" add private-data-source.json
expect_failure 'real Notion data-source identifier'
git -C "$FIXTURE_ROOT" rm -q -f private-data-source.json

uppercase_notion_id='abcdefabcdefabcdefab''cdefabcdefab'
printf 'NOTION_DATABASE_ID=%s\n' "$uppercase_notion_id" > "$FIXTURE_ROOT/private-notion.env"
git -C "$FIXTURE_ROOT" add private-notion.env
expect_failure 'uppercase Notion database identifier' "$uppercase_notion_id"
git -C "$FIXTURE_ROOT" rm -q -f private-notion.env

multiline_notion_id='1234567890abcdef1234''567890abcdef'
printf '{"data_source_id":\n  "%s"}\n' "$multiline_notion_id" > "$FIXTURE_ROOT/private-multiline-notion.json"
git -C "$FIXTURE_ROOT" add private-multiline-notion.json
expect_failure 'multiline Notion data-source identifier' "$multiline_notion_id"
git -C "$FIXTURE_ROOT" rm -q -f private-multiline-notion.json

compact_notion_id='11111111222233334444''555555555555'
printf 'database_id = "%s"\n' "$compact_notion_id" > "$FIXTURE_ROOT/private-compact-notion.toml"
git -C "$FIXTURE_ROOT" add private-compact-notion.toml
expect_failure 'compact Notion identifier'
git -C "$FIXTURE_ROOT" rm -q -f private-compact-notion.toml

notion_url="https://www.notion.so/example/${compact_notion_id}?v=aaaaaaaa"
printf '%s\n' "$notion_url" > "$FIXTURE_ROOT/private-notion-url.txt"
git -C "$FIXTURE_ROOT" add private-notion-url.txt
expect_failure 'Notion URL identifier'
git -C "$FIXTURE_ROOT" rm -q -f private-notion-url.txt

loopback='http://127.0.0.''1:4567/feed'
printf "url = '%s'\n" "$loopback" >> "$FIXTURE_ROOT/feeds.toml"
git -C "$FIXTURE_ROOT" add feeds.toml
expect_failure 'single-quoted loopback canonical feed' "$loopback"

printf '%s\n' 'public-tree scan tests passed'
