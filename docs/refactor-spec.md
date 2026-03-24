# rss-scout 重构 Spec

## 目标

将 bash+python 脚本集合重构为 **Rust CLI 二进制**。

## 现状

```
rss-scout.sh (252行 bash) → parse_feed.py (83行) → filter_entries.py (274行)
+ check-new-scout.sh + 2个 launchd workaround 脚本
```

问题：3 个文件 + provenance 限制 + bash/python 混合 + feeds 硬编码 + 无测试

## 目标结构

```
rss-scout/
├── Cargo.toml
├── feeds.toml              # 113+ 个源定义
├── src/
│   ├── main.rs             # clap CLI 入口 (~50行)
│   ├── config.rs           # feeds.toml 加载 (~60行)
│   ├── fetcher.rs          # ureq HTTP (~40行)
│   ├── parser.rs           # feed-rs 统一解析 (~30行)
│   ├── filter.rs           # 关键词+arXiv增强过滤 (~150行)
│   ├── dedup.rs            # seen.txt 去重+URL归一化+过期 (~80行)
│   └── report.rs           # Markdown 报告生成 (~50行)
└── tests/
```

## CLI 接口

```
rss-scout run [--dry-run] [--feeds PATH] [--data-dir PATH]
rss-scout check           # 检查未读报告
rss-scout feeds            # 列出所有源
```

## 依赖

```toml
feed-rs = "2.2"       # 统一 RSS/Atom/JSON Feed 解析
ureq = "3"            # 同步 HTTP (无 tokio)
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
regex = "1"
chrono = "0.4"
url = "2"
```

## 关键决策

- Rust 原生二进制 → 天然解决 macOS provenance 问题
- 报告格式不变（Knowledge Scout Skill 依赖）
- seen.txt 格式不变（639 条数据无缝继承）
- feeds 从 bash 硬编码 → feeds.toml 配置文件

## 安装

```bash
cd rss-scout && cargo install --path .
# → ~/.cargo/bin/rss-scout
```

## launchd 简化

```xml
<string>~/.cargo/bin/rss-scout</string>
<string>run</string>
```

不再需要 workaround 脚本。
