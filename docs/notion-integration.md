# Notion 日报同步

可选的 Notion 同步已接入 `rss-scout run`，默认关闭。启用后，CLI 将
P0/P1 条目写入指定 Database 的当日日报页。`--dry-run` 不调用 Notion。
当前实现不包含持久投递队列；下文说明源码行为，不代表线上交付保证。

## 页面结构

采用 **Database + Daily Summary Page 混合方案**（方案 C）：

- Notion Database 属性只有两个：`Title`（title）+ `Date`（date）
- 每天自动生成一个 Daily Summary Page，标题格式 `X月X日AI资讯`
- 页面内容按 `h2 → 描述 → 来源链接 → 图片 → 分隔线` 结构排列
- 可在 Notion 中选择 Gallery 视图；封面图取自第一条带图条目

## 配置

### feeds.toml

```toml
[notion]
enabled = true
database_id = "<your-notion-database-id>"
```

在自己的 feed 配置副本中加入该节，并替换占位 Database ID。
当前解析器支持 `[notion]`；不要将真实 ID 或 API key 提交到公开仓库。

### 环境变量

```bash
export NOTION_API_KEY="<your-notion-api-key>"
```

需要在 Notion 中创建 Integration 并授权到目标 Database，配置
`Title`（title）和 `Date`（date）属性。使用自己的配置运行：

```bash
rss-scout run --feeds /path/to/your-feeds.toml
```

启用同步后，缺少或为空的 `database_id`、`NOTION_API_KEY` 以及 Notion
请求失败都会令本次运行返回错误。CLI 在同步成功后才保存 seen 链接；
失败后可重跑，但没有后台补发队列。

## 数据流

```
RSS Scout run
  → 采集配置中的源 → 关键词过滤 → 去重 → 评分
  → 筛选 P0 + P1 条目
  → Notion API: 查询 Date 属性去重（当天已有则跳过）
  → Notion API: POST /pages 创建日报页面
  → 超过 100 blocks 时分批 PATCH 追加
```

## 页面内容格式

CLI 生成以下标题、日期和内容块：

```
┌─────────────────────────────┐
│ cover: 第一条带图条目的图片    │
├─────────────────────────────┤
│ Title: 4月3日AI资讯           │
│ Date: 2026-04-03             │
├─────────────────────────────┤
│ ## 1、文章标题                │
│                              │
│ 描述文本（HTML 清理后，≤300字）│
│                              │
│ 来源：源名称（超链接，下划线） │
│                              │
│ [图片 block（如有）]          │
│                              │
│ ───────── 分隔线 ──────────  │
│                              │
│ ## 2、下一条...               │
└─────────────────────────────┘
```

### Block 结构详解

每条条目由 3-5 个 block 组成：

1. **heading_2** — `{序号}、{标题}`
2. **paragraph** — 描述文本（strip HTML，截断 300 字符）。空描述跳过
3. **paragraph** — 来源链接：`来源：` + 源名称做超链接（underline 注解）
4. **image**（可选）— 条目有图片时内嵌 external image block
5. **divider**（非最后一条）— 条目间分隔线

### Notion API 调用

```
POST https://api.notion.com/v1/pages
Headers:
  Authorization: Bearer $NOTION_API_KEY
  Content-Type: application/json
  Notion-Version: 2022-06-28

Body:
{
  "parent": {"database_id": "..."},
  "cover": {"type": "external", "external": {"url": "封面图URL"}},
  "properties": {
    "Title": {"title": [{"text": {"content": "X月X日AI资讯"}}]},
    "Date": {"date": {"start": "YYYY-MM-DD"}}
  },
  "children": [... blocks ...]
}
```

## 去重与限制

通过查询 Database 中是否已存在当天日期的页面：

```
POST /databases/{id}/query
Body: {"filter": {"property": "Date", "date": {"equals": "YYYY-MM-DD"}}, "page_size": 1}
```

已存在当日页面时返回 `SyncOutcome::PageExists` 并跳过；没有 P0/P1
条目时返回 `SyncOutcome::NoHighPriority`。当前实现不会更新已存在的页面。
创建页面后的追加块请求若失败，已创建页面不会自动回滚；重跑可能因
当日页面已经存在而跳过，需人工检查缺失内容。

## 重试策略

- 429（Rate Limit）和 5xx 错误自动重试
- 指数退避：500ms → 1s
- 最多 3 次请求尝试（初次请求加 2 次重试）
- 网络错误同样重试

## 两种创建模式对比

| 维度 | Rust 自动推送 | `/knowledge-scout` skill |
|------|-------------|-------------------------|
| 触发 | `rss-scout run`（本仓 launchd 管线已退役） | 外部 `/knowledge-scout` |
| 内容 | 原始 RSS 标题 + 描述 | AI 中文摘要润色 |
| 图片 | RSS 原生 media:thumbnail / desc `<img>` | og:image（WebFetch 提取） |
| 格式 | 本文说明的标题、日期和内容块 | 由外部 skill 决定 |
| 条目 | P0 + P1 | AI 筛选后的精选 |

此表只区分本仓 CLI 与外部 skill 的职责。启用 `[notion]` 后，非
dry-run 的 CLI 运行会尝试同步；外部 skill 的运行状态不由本仓管理。

## 代码位置

- [`src/notion.rs`](../src/notion.rs) — NotionClient、内容块和请求重试
- [`src/config.rs`](../src/config.rs) — NotionConfig 结构体
- [`src/main.rs`](../src/main.rs) — `run()` 中的 `maybe_sync_notion` 调用
