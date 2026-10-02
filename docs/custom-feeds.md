# Use a custom RSS feed list

rss-scout turns RSS or Atom entries into a daily Markdown report. It requires
the [source installation](../README.md#install) and network access to your feeds.
This example uses the official Go blog without API keys or Notion setup.

## Start with one feed

Save this as `my-feeds.toml` in your checkout.

```toml
[settings]
keywords = ""
max_items = 10
seen_expire_days = 90

[[feeds]]
name = "Go Blog"
url = "https://go.dev/blog/feed.atom"
skip_filter = true
```

The empty keyword pattern and `skip_filter = true` keep this feed's entries
without the general AI-development keyword filter. `max_items` is the supported
global per-feed limit. Do not add feed-level adapter options to make this CLI
fetch an API; those fields are only passed through for downstream pipelines.

```bash
rss-scout feeds --feeds ./my-feeds.toml
rss-scout run --dry-run --feeds ./my-feeds.toml --data-dir ./demo-data
```

The first command should list one feed. It checks local config parsing, not
remote availability. The second writes `demo-data/output/scout-YYYY-MM-DD.md`
without persisting newly seen links. It still fetches the network and writes a
report; keep `demo-data` outside version control. Repeating a run on the same
day uses that day's report path.

After checking the report, remove `--dry-run` to persist seen links and avoid
reporting them as new on later runs. Use a separate data directory while trying
new configurations so you do not mix a trial with an existing seen store.

## Import OPML from an RSS reader

Export OPML from your reader to `subscriptions.opml`, then preview the import
against the same config file.

```bash
rss-scout import --dry-run --feeds ./my-feeds.toml subscriptions.opml
rss-scout import --feeds ./my-feeds.toml subscriptions.opml
rss-scout feeds --feeds ./my-feeds.toml
```

Import probes candidate feeds over the network and skips domains already in
the config. Dry run prints what it would append without changing the TOML.
Review the preview before the second command, especially if you expect several
feeds from a domain you already subscribe to.

## Why is the report empty?

Read fetch errors in the terminal and confirm that the feed URL still returns
RSS or Atom. For feeds with `skip_filter = false`, `settings.keywords` is a
regular expression matched against title and description. Previously seen
links are removed even during dry run if you reuse an existing data directory.

No keys are needed for feed collection and Markdown output. The optional Notion
integration requires its own API key when enabled; dry run does not sync it.
See the [command reference](../README.md#commands),
[data files](../README.md#data-files), and
[support instructions](../README.md#support). The source package version and
MIT license are recorded in [Cargo.toml](../Cargo.toml) and [LICENSE](../LICENSE).
