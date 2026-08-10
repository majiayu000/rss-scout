# Multi-AI Research：tool-scout 工具发现改进

> **研究问题**：如何更广泛、更高频地发现适合 Claude Code / Codex / Cursor 用户的工具/skill/MCP server/AI agent 库？
>
> **触发**：tool-scout 首次跑（2026-05-08）只入选 10 条，且其中 2 条已知（mcp/servers + rtk），用户认为发现太少不够广。
>
> **耗时**：~12 分钟（4 任务并行 5-10 分钟 + Phase 5/6 整理 ~5 分钟）
>
> **任务数**：4 个并行（2 internal agents + grok + gemini）+ Phase 5.5 二次验证 8 个 gh api 调用

---

## Phase 1: 问题分解

### 内部任务（2 agent）
- **Internal A**：审查 `tool-scout.md` + `feeds-tools.toml` + `2026-05-08.md` 找具体可改进点（4 类：数据源 / 发现机制 / 评分 / fetch 上限）
- **Internal B**：用 `gh search` 找现成"AI dev tool 发现器"开源项目（4 方向：tool radar / MCP discovery / GH trending API / awesome-list 自动化）

### 外部任务（2 AI）
- **Grok** + **Gemini** 共 5 子问题（一致 prompt）：
  1. 2026 年发现渠道（X 账号 / aggregator / newsletter）
  2. MCP / Skill marketplace API
  3. 现成开源工具发现器项目
  4. 不依赖 RSS 的主动数据驱动路径
  5. 1000+ 用户级 newsletter backbone 设计

---

## Phase 5: 交叉验证矩阵

| 发现 | Internal A | Internal B | Grok | Gemini | Phase 5.5 二次验证 | Tier |
|---|---|---|---|---|---|---|
| 官方 MCP Registry API（registry.modelcontextprotocol.io） | — | ✅ 6784 stars 实测 | ✅ 详细 endpoint | — | ✅ HTTP 200 + 21KB JSON | 🟢 强共识（事实 verified） |
| Smithery.ai 公开 API + llms.txt | — | — | ✅ + CLI 命令 | ✅ llms.txt | ✅ HTTP 200 + 11KB | 🟢 强共识（双 AI + 实测） |
| ComposioHQ/awesome-claude-skills | — | — | ✅ 1000+ skills | — | ✅ **58627 stars** 实测 | 🟢 单 AI + gh 验证 |
| travisvn/awesome-claude-skills | — | — | ✅ | — | ✅ 12256 stars 实测 | 🟢 单 AI + gh 验证 |
| ai-for-developers/awesome-ai-coding-tools | — | — | ✅ "1.7k stars" | — | ✅ 1703 stars 实测 | 🟢 单 AI + gh 验证（数字 ±5%） |
| appcypher/awesome-mcp-servers | — | ✅ "5.5k" | ✅ 间接 | — | ✅ 5525 stars 实测 | 🟢 双源共识 |
| punkpeye/awesome-mcp-servers (已订阅) | — | ✅ 86453 stars | — | — | ✅ 已在 feeds.toml | 🟢 已采用 |
| GitHub Search API by topic 主动发现 | ✅ 详细命令 | — | ✅ 详细 | ✅ 详细 | — | 🟢 三家强共识 |
| npm/PyPI/cargo 增量流 | ✅ 详细 endpoint | — | ✅ 详细 | ✅ 详细 | — | 🟢 三家强共识 |
| GitHub trending 没官方 API（事实） | — | ✅ gh search 验证 5 个 scrape 项目 | — | — | — | 🟢 事实层 verified |
| GH-Trend 应加 8 个新语言 | ✅ 论证 C/Zig/Lua/Swift/Kotlin/Shell/HTML | — | — | — | — | 🟡 单家但论证强 |
| ShowHN 关键词太窄 → 加全量 ShowHN | ✅ 推荐 hnrss.org/show?count=50 | — | — | — | — | 🟡 单家但实施成本低 |
| GH releases 18 源 = 已知池闭环 | ✅ 详细论证 | — | — | — | — | 🟡 单家论证强 |
| Awesome list 自动监控空白赛道 | — | ✅ gh search 多次证空 | — | — | — | 🟢 事实（双内部 + gh 验证）|
| pre-rank Q 维度提前算（cold-start 兜底） | ✅ 完整设计 | — | ✅ 间接（"先 LLM 过滤"） | ✅ 间接（Refinement 层） | — | 🟢 三家共识 |
| GitPulse "growth multiples" 评分 | — | ✅ 代码细节 | — | — | — | 🟡 单家但实测代码 |
| N 维 grep 扩范围到 docs/{tools,rss} | ✅ 详细论证 | — | — | — | — | 🟢（用户已提 + Internal A） |
| **❌ duanyytop/agents-radar 12k stars** | — | ❌ 没找到 | — | ✅ 唯一来源 | ⚠️ **真存在但 724 stars**（数字 16× 幻觉） | 🚫 数字幻觉 |
| **❌ mcp-get/mcp-get 8.5k stars** | — | ❌ 没找到（找到 mcpm.sh 940 stars） | — | ✅ 唯一来源 | ❌ **404 完全幻觉** | 🚫 hallucination |
| **❌ cursor-tools/awesome-cursor-rules 25k** | — | — | — | ✅ 唯一来源 | ❌ **404 完全幻觉**（真名 PatrickJS/awesome-cursorrules） | 🚫 hallucination |
| **❌ ClawHub / clawhub.dev/api/v1/skills** | — | ❌ 没找到 | — | ✅ 唯一来源 | — | 🚫 单家可疑 |
| **❌ Phoenix 算法**（X 算法）| — | — | — | ✅ Gemini 反复出现 | 历史已知 hallucination（2026-04-08 案例） | 🚫 已知 hallucination |
| **❌ X 账号 @MCP_Universe @AgenticDevs @CursorHub** | — | — | — | ✅ 唯一来源 | （未验证 — Gemini X 账号常幻觉，X 又 402 无法直查） | 🚫 单家可疑 |
| **❌ wong2/awesome-mcp-servers 4k stars** | — | — | ✅ 唯一来源 | — | ❌ **404** | 🚫 hallucination（grok 单家） |
| **❌ @steipete (OpenClaw 作者)** | — | — | ✅ 唯一来源 | — | 我们 5-7 数据：实际是 RightNow-AI/openfang | 🚫 grok 幻觉 |
| MCP discovery 已超饱和（archestra/metamcp/mcpm.sh/docker） | — | ✅ 5+ 项目实测 | — | — | — | 🟢 事实（不要重造） |

### 关键发现

1. **Gemini 这次仍出现多处具体 repo / 数字幻觉**（mcp-get / cursor-tools / agents-radar 数字 16× / ClawHub / Phoenix 算法）— 与 2026-04-08 历史案例同模式。**Gemini 给的具体名词必须 fact-check**。
2. **Grok 准确率高但仍有 1 处 wong2 幻觉**。
3. **registry.modelcontextprotocol.io 公开 API 是 backbone 级数据源**（实测 21KB JSON 一次拉到全部 MCP servers）— Grok 主推 + 实测可用，应优先接入。
4. **Awesome list 自动监控是空白赛道**（Internal B 多次搜索 + 唯一同方向项目 ColinEberhardt/awesome-lists-bot 4 stars 已 10 年废弃）— tool-scout 可独占。

---

## Phase 6: Tiered Action Items

### 🔴 Tier 1（极高置信度，立刻做）

#### 1.1 接入官方 MCP Registry API（最高 ROI）
- **依据**：Grok 详细 + Internal B 实测 6784 stars + Phase 5.5 实测 HTTP 200 + 21KB JSON
- **实施**：加 feed
  ```toml
  [[feeds]]
  name = "MCP-Registry:official"
  url = "https://registry.modelcontextprotocol.io/v0/servers"
  tier = "core"
  kind = "registry-api"
  ```
- **特殊处理**：返回是 JSON 不是 RSS XML — 需要给 Step 1 解析增加 JSON 分支（按 `servers[].name + .description + .repository.url` 拼成 entry）
- **预期**：每天发现 5-15 个**新注册的 MCP server**（registry 的优先级远高于 trending）

#### 1.2 接入 Smithery.ai
- **依据**：双 AI 共识 + Phase 5.5 实测 HTTP 200
- **实施**：先用 `https://smithery.ai/docs/llms.txt`（11KB markdown）观察一周看更新频率，再决定是否升级到 registry API
- **Verify**：1 周后 diff 看 llms.txt 是否每日有新条目

#### 1.3 加 4 个高价值 awesome list（已 fact-check）
```toml
[[feeds]]
name = "Awesome:claude-skills-composio"
url = "https://github.com/ComposioHQ/awesome-claude-skills/commits.atom"  # 58k stars
kind = "awesome-list"

[[feeds]]
name = "Awesome:claude-skills-travisvn"
url = "https://github.com/travisvn/awesome-claude-skills/commits.atom"  # 12k stars
kind = "awesome-list"

[[feeds]]
name = "Awesome:mcp-appcypher"
url = "https://github.com/appcypher/awesome-mcp-servers/commits.atom"  # 5.5k stars
kind = "awesome-list"

[[feeds]]
name = "Awesome:ai-coding-tools"
url = "https://github.com/ai-for-developers/awesome-ai-coding-tools/commits.atom"  # 1.7k stars
kind = "awesome-list"
```
- **预期**：4 个新源约 +20-40 entries/day，且 Composio 58k stars 是 Claude skill 发现最大入口

#### 1.4 N 维 grep 范围扩展（解决跨 skill 失明）
- **依据**：用户已提 + Internal A 详细
- **实施**：改 1 行
  ```python
  # 旧
  hist_files = glob(f"{tool_docs}/2026-*.md")
  # 新
  hist_files = glob(f"{tool_docs}/2026-*.md") + glob(f"{rss_docs}/2026-*.md")
  ```
- **预期**：mcp/servers / rtk 这种"在 knowledge-scout 已介绍"的工具 N 直接降到 4 或 3

#### 1.5 core_terms 必含 owner/repo token + 边界匹配
- **依据**：Internal A 论证假阴/假阳率
- **实施**：改 GPT 提 core_terms 的 prompt — "至少 1 个 GitHub `owner/repo` 形式的 token"，grep 用 `-P '\b<term>\b'`

#### 1.6 加全量 ShowHN feed
- **依据**：Internal A "5 个关键词系统性遗漏"
- **实施**：加 `https://hnrss.org/show?count=50`（无关键词过滤，由 GPT 粗筛兜底）

**Tier 1 综合 rollback**：每条改动独立可 revert（feed 删除 / 1 行 grep / GPT prompt 改），出错不会传染。

**Tier 1 综合 verify**：明天跑一次 tool-scout 看：
- 入选条数（预期 15-25，比今天 10 多）
- N 分布（预期出现 N=3/4 而非全 5）
- mcp/servers / rtk 这类 knowledge-scout 已介绍工具 N≤2 不再入选

---

### 🟡 Tier 2（验证后做）

#### 2.1 GitHub Search API by topic 主动发现（突破 RSS 被动模型）
- **依据**：三家共识（Internal A + Grok + Gemini）
- **实施草稿**：
  ```python
  topics = ["mcp", "claude-code", "coding-agent", "ai-agent",
            "llm-tools", "cursor-rules", "agent-framework", "mcp-server"]
  for t in topics:
      gh api search/repositories \
        -f q="topic:$t pushed:>=$(date -v-1d) stars:>=10" \
        -f sort=stars
  ```
- **风险**：GitHub API rate limit 5000/h（认证后）— 8 topic × 1 query = 8 calls，无压力
- **何时启用**：先观察 Tier 1 一周；如果 Tier 1 不够再上

#### 2.2 npm / PyPI / cargo 增量流
- **依据**：三家共识
- **实施**：
  ```toml
  [[feeds]]
  name = "PyPI:updates"
  url = "https://pypi.org/rss/updates.xml"  # 最近 40 个上传

  [[feeds]]
  name = "Crates:newest"
  url = "https://crates.io/api/v1/crates?sort=new&per_page=50"  # JSON

  # npm 没原生 RSS，可用第三方桥接（如 npm-feed）或 search API
  ```
- **风险**：信噪比低（PyPI 一天上千个上传），需要更严格 GPT 粗筛 prompt
- **何时启用**：Tier 1 后看是否漏 day-0 工具，再决定

#### 2.3 pre-rank Q 维度提前算（cold-start 兜底）
- **依据**：Internal A 设计
- **实施**：Step 3 重构 — 对 GPT≥12 全池先算 Q（gh api 仅 metadata 调用快），再用 `0.4 GPT + 0.3 Q + 0.3 N` 做 pre-rank 选 fetch top
- **成本**：每天约 50-100 个 gh api 调用（5000/h 限额无压力）
- **风险**：会让有 GitHub 仓库的工具系统性占优（npm 包 / SaaS 没 GH 信号），需观察平衡

#### 2.4 GitPulse "growth multiples" 评分（stars today / 历史均值）
- **依据**：Internal B 代码级阅读 GitPulse 实现
- **实施**：本地缓存 candidate repos 的 stars 历史，每次 fetch 时算 `growth_today / mean_growth_30d`
- **何时启用**：Tier 2.3 上线后再叠加

#### 2.5 8 个新语言 GH-Trend
- **依据**：Internal A 论证遗漏 C/Zig/Lua/Swift/Kotlin/Shell/HTML
- **实施**：加 8 个 mshibanami.github.io feed（已知 URL 模板）
- **何时启用**：观察 Tier 1+1.6（全量 ShowHN）够不够，不够再加

#### 2.6 删除 G 类 14 个低发现价值 release feeds
- **依据**：Internal A 论证强（已知工具版本流不能产生 N≥4 候选）
- **保留**：claude-code / openai-codex / cursor / zed-industries 4 个旗舰
- **删除**：aider / continue / cline / goose / baml / mcp-servers / tensorzero / agent-browser / open-agents / deer-flow / rtk / opencode / acpx / openfang
- **替代**：这些工具的"新 release"会经由 awesome list commits / 社区 / Show HN 自然出现
- **风险**：可能漏掉某些工具的重大 release（如 v2.0 突破性升级）— **保守做法是先观察再删**
- **何时启用**：Tier 2.1 GitHub Search 上线后，观察 awesome+search 是否覆盖 release 信号

#### 2.7 Awesome list 自动监控（独占赛道）
- **依据**：Internal B 多次搜索证空 + ColinEberhardt 项目 10 年废弃
- **实施**：每天 diff `awesome-mcp-servers/README.md` URL 集合 vs 昨天，新增的 URL = 新工具
- **价值**：commits.atom 漏了"压缩 commit / squash" 的批量加入；URL diff 是补漏机制
- **何时启用**：Tier 1.3 加 4 个 awesome 后，观察 commits.atom 是否漏抓，漏多再上

---

### ⚪ Tier 3（仅当 Tier 1+2 明显不够再做）

| # | Action | 理由（为什么放后面） |
|---|---|---|
| 3.1 | archestra/metamcp/mcpm.sh/docker-mcp-registry 集成 | MCP discovery 已饱和（Internal B 6 个项目）— 消费 1.1 官方 registry 已够 |
| 3.2 | GitHub stars-velocity 24h 增速追踪 | 需建 daily snapshot db，工程复杂度高，Tier 2.4 GitPulse 模式更轻量 |
| 3.3 | mcp.so 数据源 | Grok 单家提（"2万+ servers"），未实测 API；先用 1.1 官方 registry |
| 3.4 | NiklasTiede / huchenme GH-Trend API | 现 GH-Trend RSS 已经够用，加 API 调用是过度工程 |
| 3.5 | LLM 多 agent 分类去重（Gemini "Refinement" 层）| 当前 Claude N/F/Q 已是分类机制，不必引入二级 agent |
| 3.6 | Bluesky firehose 替代 X | X 已 402 无法替代，Bluesky 用户量不足；先看 awesome list 够不够 |

---

### 🚫 不做（验证失败 / 单家潜在幻觉）

| # | Claim | 来源 | 失败证据 |
|---|---|---|---|
| ❌ | `mcp-get/mcp-get` 8.5k stars | Gemini 唯一 | gh api 404 |
| ❌ | `cursor-tools/awesome-cursor-rules` 25k | Gemini 唯一 | gh api 404（真名 PatrickJS/awesome-cursorrules 已订阅） |
| ❌ | `duanyytop/agents-radar` 12k stars | Gemini 唯一 | 真存在但 **724 stars**（数字 16× 幻觉，存疑工具不值得加） |
| ❌ | `ClawHub` clawhub.dev | Gemini 唯一 | gh search 没找到 |
| ❌ | `wong2/awesome-mcp-servers` 4k | Grok 唯一 | gh api 404 |
| ❌ | X 账号 `@MCP_Universe` `@AgenticDevs` `@CursorHub` | Gemini 唯一 | 历史 Gemini X 账号高频幻觉 + X 已 402 无法 quick verify |
| ❌ | `@steipete (OpenClaw 作者)` | Grok 唯一 | 实际是 RightNow-AI/openfang（5-7 已知） |
| ❌ | Phoenix 算法（X 算法）| Gemini 唯一 | 2026-04-08 历史已知 hallucination |

---

## Phase 7: Artifact 元数据

- **路径**：`/Users/lifcc/Desktop/code/AI/tools/rss-scout/docs/artifacts/multi-ai-research-tool-scout-improve-20260508-110000.md`
- **触发**：用户 2026-05-08 调用 `multi-ai-research` skill
- **总耗时**：~12 分钟
- **任务数**：4 并行 + 8 fact-check API
- **下游**：Tier 1 改动 5 项可在 30 分钟内全部落地，明天 tool-scout 跑可验证

---

## 下一步建议（给用户）

最小落地路径（1 小时内完成）：
1. **加 4 个 awesome list feed**（5 分钟，Tier 1.3）
2. **N 维 grep 扩范围**（1 行修改，Tier 1.4）
3. **加全量 ShowHN feed**（1 行修改，Tier 1.6）
4. **接入官方 MCP Registry API**（Step 1 加 JSON 分支，~30 分钟，Tier 1.1）

接 1.2 Smithery 和 Tier 2 全部留下次迭代。

明天跑 tool-scout 验证：
- 入选条数应从 10 → 15-25
- N 分布出现 N=3/4 而非全 5
- mcp/servers / rtk 这种 knowledge-scout 已介绍工具不再入选
- 应该出现的新工具：MCP Registry 的 day-0 注册、Composio awesome 的 PR 推荐、全量 Show HN 中之前漏的非热词工具
