# Notion 集成设计草案（未实现）

> 状态：仅作为未来设计参考。当前公开树中没有 Notion client、
> durable delivery 状态机或对应的 CLI 配置支持；下文描述的是拟议行为，
> 不是已发布能力。

## 拟议架构

采用 **Database + Daily Summary Page 混合方案**（方案 C）：

- Notion Database 属性只有两个：`Title`（title）+ `Date`（date）
- 每天自动生成一个 Daily Summary Page，标题格式 `X月X日AI资讯`
- 页面内容按 `h2 → 描述 → 来源链接 → 图片 → 分隔线` 结构排列
- Gallery 视图展示，封面图取自第一条带图条目

## 配置

### feeds.toml

```toml
[notion]
enabled = true
database_id = "<your-notion-database-id>"
```

此片段仅描述未来集成方向。当前 `rss-scout` 配置解析器会拒绝
`[notion]`，因此不要将此段复制到规范 `feeds.toml` 或
`feeds-tools.toml`。Notion 持久投递尚未包含在当前 CLI 中。

### 环境变量

```bash
export NOTION_API_KEY="ntn_xxx"  # 写入 ~/.zshrc
```

需要在 Notion 中创建 Integration 并授权到目标 Database。

## 拟议数据流

```
RSS Scout run
  → 采集 113 源 → 关键词过滤 → 去重 → 评分
  → 筛选 P0 + P1 条目
  → Notion API: 查询 Date 属性去重（当天已有则跳过）
  → Notion API: POST /pages 创建日报页面
  → 超过 100 blocks 时分批 PATCH 追加
```

## 拟议页面内容格式

与 `/knowledge-scout` skill 手动创建的格式一致：

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

## 拟议去重

通过查询 Database 中是否已存在当天日期的页面：

```
POST /databases/{id}/query
Body: {"filter": {"property": "Date", "date": {"equals": "YYYY-MM-DD"}}, "page_size": 1}
```

未来实现可在已存在时跳过，并返回 `Ok(false)`。

## 拟议重试策略

- 429（Rate Limit）和 5xx 错误自动重试
- 指数退避：500ms → 1s → 2s
- 最多 3 次重试
- 网络错误同样重试

## 两种创建模式对比

| 维度 | Rust 自动推送 | `/knowledge-scout` skill |
|------|-------------|-------------------------|
| 触发 | launchd 定时 + `rss-scout run` | 手动 `/knowledge-scout` |
| 内容 | 原始 RSS 标题 + 描述 | AI 中文摘要润色 |
| 图片 | RSS 原生 media:thumbnail / desc `<img>` | og:image（WebFetch 提取） |
| 格式 | 完全一致 | 完全一致 |
| 条目 | P0 + P1 | AI 筛选后的精选 |

此表仅比较拟议的 Rust 自动推送与现有手动 skill 流程；
当前 `rss-scout` 不会自动推送到 Notion。

## 拟议代码位置

- `src/notion.rs` — 未来的 NotionClient + block 构建
- `src/config.rs` — 未来的 NotionConfig 结构体
- `src/main.rs` — 未来在 `run()` 末尾接入的 Notion sync 逻辑
