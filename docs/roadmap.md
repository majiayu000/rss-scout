# rss-scout Roadmap

## Phase 1: 核心质量（评分 + 报告分层）

解决最大痛点：500 行扁平报告 68% 是版本号噪音。

### 1.1 feeds.toml 扩展

给每个 feed 加 `tier` 和 `kind` 字段：

```toml
[[feeds]]
name = "SimonWillison"
url = "https://simonwillison.net/atom/everything/"
tier = "core"       # core(3) | official(2) | aggregator(1) | release(0)
kind = "blog"       # blog | changelog | news | podcast | github

[settings.scoring]
keywords_high = ["claude code", "mcp", "agentic coding", "guardrail", "swe-bench", "alignment", "context engineering"]
keywords_mid = ["agent", "benchmark", "evaluation", "fine-tuning", "rag", "code review", "rust", "cargo"]
```

### 1.2 新建 scorer.rs

四维评分（每维 0-3，总分 0-12）：
- 源权重：core(3) > official(2) > aggregator(1) > release(0)
- 关键词密度：high 命中 2+→3, 1→2; 仅 mid 命中→1
- 新鲜度：24h(3) > 48h(2) > 72h(1) > 更早(0)
- 独特性：域名在 seen.txt 中出现次数 ≤2→3, ≤10→2, ≤30→1, >30→0

输出 P0(≥9) / P1(6-8) / P2(<6) 标签。

### 1.3 report.rs 分层渲染

```markdown
## 今日亮点 (T1 条目标题列表)

## P0 高价值 (3 条)
- **[12] Claude Code gets MCP filtering** — simonwillison.net
  源:3 词:3 鲜:3 独:3

## P1 值得关注 (8 条)
...

<details><summary>P2 低优先级 (15 条)</summary>
...
</details>

## 版本追踪
- **Claude Code**: v2.1.75~v2.1.81 (7 releases)
- **Windsurf**: 1.9544~1.9577 (30 releases)
- **Cursor**: 2.4.23~2.6.21 (20 releases)
```

Changelog 压缩：同 feed 多条版本号合并为 1 行（范围+数量+亮点摘要）。

### 1.4 main.rs 两阶段分离

```
阶段 1: collect — for each feed → fetch/parse/filter/dedup → Vec<ScoredEntry>
阶段 2: render — sort by score → write_summary → write_tiered → write_changelog
```

**改动文件**：config.rs(+15行) + scorer.rs(新建~120行) + report.rs(+80行) + main.rs(改~30行) + feeds.toml(加 tier/kind)

---

## Phase 2: 性能 + 源扩展

### 2.1 rayon 并行采集

```toml
# Cargo.toml
rayon = "1"
```

核心改动：`cfg.feeds.iter()` → `cfg.feeds.par_iter()`

两阶段设计：
- 并行阶段：fetch + parse + keyword filter（不访问 SeenDb）
- 串行阶段：dedup + score + report write

预期：1.5 分钟 → 15-25 秒。discover 子命令同样受益。

**改动文件**：Cargo.toml(+1行) + main.rs(改~30行) + discover.rs(改~5行)

### 2.2 新增 14 个高价值源

来自 OPML 调研和 discover 自动发现：

| 源 | URL | tier |
|---|---|---|
| Apple ML | `machinelearning.apple.com/feed.xml` | official |
| Amazon Science | `amazon.science/index.rss` | official |
| AI Snake Oil | `aisnakeoil.substack.com/feed` | core |
| The Gradient | `thegradient.pub/rss/` | core |
| Semgrep Blog | `semgrep.dev/blog/rss` | core |
| JetBrains | `blog.jetbrains.com/feed/` | official |
| Windsurf Blog | `windsurf.com/feed.xml` | official |
| Cursor Blog | `cursor.com/atom.xml` | official |
| Cursor Changelog | `cursor-changelog.com/feed` | release |
| Confident AI | `confident-ai.com/feed.xml` | core |
| fast.ai | `fast.ai/atom.xml` | core |
| Nicholas Carlini | `nicholas.carlini.com/writing/feed.xml` | core |
| Dwarkesh Blog | `dwarkesh.com/feed` | core |
| Slashdot | `rss.slashdot.org/Slashdot/slashdotMain` | aggregator |

**改动文件**：feeds.toml（+14 条）

### 2.3 OPML 导入子命令

```
rss-scout import <file.opml> [--dry-run]
```

解析 OPML XML → 去重（域名级）→ 验证可用性 → 追加到 feeds.toml。

**新建文件**：src/opml.rs(~60行) + main.rs 加 Import subcommand

---

## Phase 3: 健壮性

### 3.1 discover 模块修复

- 正则提升为 `LazyLock` 静态编译（消除每次调用重编译）
- `fetch_text` 删除，复用 `fetcher::fetch`
- HTML autodiscovery 改为松散匹配（不依赖属性顺序）
- SKIP_DOMAINS 补全（+substack.com, facebook.com, instagram.com, spotify.com 等）
- PROBE_PATHS 补充（/feed/rss, /feeds, /articles.rss, /posts.rss）
- fetcher.rs 加 `timeout_connect(5s)`

### 3.2 测试

- scorer.rs: 评分边界值测试
- filter.rs: arXiv 三层过滤 TP/FP 测试
- dedup.rs: URL 归一化 + 过期清理测试
- discover.rs: autodiscovery 正则匹配测试
- parser.rs: RSS/Atom/JSON Feed 解析测试

### 3.3 旧文件清理

确认 Rust 版本稳定运行 1 周后：
- 删除 rss-scout.sh, parse_feed.py, filter_entries.py, check-new-scout.sh
- 删除 ~/.rss-scout/run.sh, ~/.rss-scout/scout-launcher.sh
- 更新 README

---

## 时间线

| Phase | 内容 | 预估 |
|-------|------|------|
| 1 | 评分 + 报告分层 | 1 session |
| 2 | 并行 + 新源 + OPML | 1 session |
| 3 | 健壮性 + 测试 + 清理 | 1 session |

## 不做的事

- 不用 LLM 评分（纯规则足够，零成本）
- 不发 crates.io/PyPI（个人工具）
- 不引入 tokio（rayon 足够）
- 不改 seen.txt 格式（639 条数据兼容）
- 不改报告尾部统计行格式（check 子命令依赖）
