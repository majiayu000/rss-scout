# rss-scout

rss-scout is a local, zero-API-key RSS discovery CLI that scans configured AI-development feeds, filters and scores new entries, and writes a daily Markdown report.

## Install

Requirements:

- Current stable Rust toolchain with Cargo (the project does not declare a separate MSRV)
- Network access for `run`, `discover`, and OPML `import` feed validation

Install from this source checkout:

```bash
git clone https://github.com/majiayu000/rss-scout.git
cd rss-scout
cargo install --path .
```

Or run without installing:

```bash
cargo run --quiet -- --help
```

The CLI defaults to `$HOME/.rss-scout/feeds.toml`. From a fresh source checkout, pass the checked-in config explicitly:

```bash
rss-scout feeds --feeds ./feeds.toml
rss-scout run --dry-run --feeds ./feeds.toml --data-dir /tmp/rss-scout-demo
```

For macOS launchd scheduling, generate a user-specific LaunchAgent instead of installing the template plist directly:

```bash
./install-launchd.sh --feeds ./feeds.toml
```

Use `./install-launchd.sh --help` for `--data-dir`, `--bin`, `--plist`, `--label`, `--load`, and dry-run options.

> **Note (2026-08):** This repo's launchd pipeline has been retired from production — daily scheduling now runs via the external `knowledge-scout` runtime, which consumes this repo's `feeds.toml`. Do not install this LaunchAgent alongside it (it would cause double scheduling and double writes to the same seen store). The install script is kept for reference only.

## Quickstart

```bash
cargo run --quiet -- --help
cargo run --quiet -- feeds --feeds ./feeds.toml | sed -n '1,5p'
```

The first command prints:

```text
Zero-API knowledge discovery for AI dev

Usage: rss-scout <COMMAND>

Commands:
  run       Fetch, filter, and generate report
  check     Check for unread reports
  feeds     List all configured feeds
  discover  Auto-discover new RSS feeds from recent report links
  import    Import feeds from OPML file
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

The second command starts with:

```text
  1. [skip] [aggregator/news] HN:agentic-coding — https://hnrss.org/newest?q=agentic+coding&points=5&count=30
  2. [skip] [aggregator/news] HN:vibe-coding — https://hnrss.org/newest?q=vibe+coding&points=5&count=20
  3. [skip] [aggregator/news] HN:AI-coding-agent — https://hnrss.org/newest?q=AI+coding+agent&points=10&count=20
  4. [skip] [aggregator/news] HN:claude-code — https://hnrss.org/newest?q=claude+code&points=5&count=20
  5. [skip] [aggregator/news] HN:context-engineering — https://hnrss.org/newest?q=context+engineering&points=5&count=20
```

To generate a report, run:

```bash
rss-scout run --dry-run --feeds ./feeds.toml --data-dir /tmp/rss-scout-demo
```

Sample output shape:

```text
[HH:MM:SS] 开始并行采集...
[HH:MM:SS] 完成: N 新 / M 总
[HH:MM:SS] 报告: /tmp/rss-scout-demo/output/scout-YYYY-MM-DD.md
/tmp/rss-scout-demo/output/scout-YYYY-MM-DD.md
```

`--dry-run` still writes the report file, but it does not save new entries to `seen.txt`.

## Commands

```text
rss-scout run [--dry-run] [--feeds PATH] [--data-dir PATH]
rss-scout check [--data-dir PATH]
rss-scout feeds [--feeds PATH] [--data-dir PATH]
rss-scout discover [--url URL] [--days N] [--feeds PATH] [--data-dir PATH]
rss-scout import [--dry-run] [--feeds PATH] [--data-dir PATH] <FILE>
```

- `run` fetches configured feeds, filters entries, scores them, writes `output/scout-YYYY-MM-DD.md`, and saves seen links unless `--dry-run` is set.
- `check` prints an unread-report notification only when the latest report has a nonzero new-entry count and differs from `.last-read`.
- `feeds` lists configured feed names, filter mode, tier, kind, and URL.
- `discover --url URL` probes one site for an RSS or Atom feed. Without `--url`, `discover` scans recent reports for external links and prints candidate feed snippets.
- `import <FILE>` extracts feeds from OPML, skips already-known domains, validates candidates over the network, and appends valid feeds unless `--dry-run` is set.

## Proof

Real no-network proof from this checkout:

```bash
target/debug/rss-scout feeds --feeds feeds.toml | tail -n 1
```

Output:

```text
共 199 个源
```

Real help proof from this checkout:

```bash
target/debug/rss-scout --help
```

Output includes:

```text
Commands:
  run       Fetch, filter, and generate report
  check     Check for unread reports
  feeds     List all configured feeds
  discover  Auto-discover new RSS feeds from recent report links
  import    Import feeds from OPML file
```

Sample network report output is shown in Quickstart because item counts and feed availability vary by date and network state.

## Data Files

By default, runtime data lives under `$HOME/.rss-scout`:

- `feeds.toml`: feed configuration, unless `--feeds PATH` is passed
- `seen.txt`: normalized seen-link database
- `output/scout-YYYY-MM-DD.md`: generated Markdown reports
- `.last-read`: marker used by `check`

The checked-in configs are portable examples that parse without network access:

- `feeds.toml` is the general AI-development source list used by the examples.
- `feeds-tools.toml` is a tool-discovery source list with only public URLs.

Validate either config before running a network fetch:

```bash
cargo run --quiet -- feeds --feeds ./feeds.toml
cargo run --quiet -- feeds --feeds ./feeds-tools.toml
```

Both files use only fields implemented by the current parser. Feed-level
`adapter`, `adapter_params`, and `max_items` settings are future design ideas;
the supported item limit is the global `[settings].max_items` value.

## Limitations And Caveats

- `run`, `discover`, and OPML `import` depend on live HTTP requests. Feed outages, timeouts, and site blocking can change results.
- `feeds` lists local config only. It does not validate remote feed health.
- `--dry-run` prevents seen-link persistence, but it still creates the output directory and report file.
- The current CLI does not implement `--version`; the source package version is in `Cargo.toml`.
- Optional `[notion]` section enables a daily summary sync to a Notion database (`NOTION_API_KEY` env var required); any failure fails the run explicitly. See `docs/notion-integration.md`.
- Feed-specific adapter fields (`adapter`, `adapter_params`, `max_items`, `host_min_interval_seconds`) are accepted for downstream pipeline compatibility but not implemented by this CLI.
- Default feed discovery uses `$HOME/.rss-scout/feeds.toml`, so installed users should pass `--feeds` or copy the config there.

## Repository Verification

The repository CI audits `Cargo.lock` with pinned `cargo-audit` 0.22.2, runs
formatting, Clippy with warnings denied, the full Rust test suite, both
checked-in config parses, shell syntax checks, `git diff --check`, and a
byte-level scan of every tracked file. Run the public-tree gate locally with:

```bash
./scripts/check-public-tree.sh
bash tests/public-tree-scan.sh
```

## Release And Package Status

`Cargo.toml` identifies this package as `rss-scout` version `0.1.0` under the MIT license. This repository documents source installation with `cargo install --path .`; no crates.io, Homebrew, PyPI, or binary release workflow is documented in this checkout.

## Support

Use GitHub issues for bugs and launch-readiness gaps:

https://github.com/majiayu000/rss-scout/issues

Include the command, `--feeds` path, `--data-dir` path, relevant stderr/stdout, OS, and whether the run used `--dry-run`.
