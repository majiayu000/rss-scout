# Multi-AI Research: Knowledge Scout 改进方案

**研究问题**: CyrilXBT 的 Obsidian+Claude 内容创作系统（6 阶段）哪些方法论可迁移到 Knowledge Scout 知识发现系统？

**日期**: 2026-04-13 ~ 2026-04-14
**并行任务**: 3（Agent A 内部架构分析 + Grok + Gemini）
**最终状态**: 3/3 成功 + 实验验证

---

## 最终架构决策（2026-04-14 实验验证）

### 三层架构

```
Layer 1 (代码, ~30s): curl 并行下载 + Python XML 解析 + seen.txt 去重
Layer 2 (GPT-5.4, ~90s): 10 路并发语义粗筛, 阈值 ≥8, ~300-500 条通过
Layer 3 (Claude, ~5-10min): 精筛 + WebFetch 深度分析 + Connection + Notion
```

### 实验数据支撑

**三方对比**（2511 条输入，105 条 Claude 高价值基准）：

| 方案 | 对 Claude 高价值覆盖率 | 耗时 | Claude 用量 |
|---|---|---|---|
| 代码关键词预过滤 | 62.9% ❌ 漏杀 39 条 | <1s | 1145 条 |
| GPT-5.4 串行 | 90.5%（实际 ~98%） | 32min | ~100 条 |
| GPT-5.4 并发×10 | 90.5%（实际 ~98%） | **93s** | ~100 条 |
| 纯 Claude 5 agent | 100% | ~5min | 2511 条 |

**关键发现**：
- GPT-5.4 的 10 条"漏杀"中 **7 条是 link 匹配问题**（同文章不同 URL），非评分问题
- GPT 给 Anthropic 官博 18-20 分，没有竞品低估
- 按标题匹配后真实漏杀仅 ~3 条，覆盖率 ~98%
- 并发 10 路：93s vs 串行 1907s，**20.5x 加速**
- 代码关键词过滤无法捕捉"Is the IDE dead?"等语义相关条目

### Prompt 修正

GPT-5.4 唯一真实盲区：知名工程师的非 AI 标题文章（如 Mitchell Hashimoto 14 分 vs Claude 16 分）。
修正方案：prompt 加 "AI 领域知名工程师文章默认 +3 分" + 阈值从 ≥10 降到 ≥8。

---

## 研究来源

CyrilXBT (@cyrilXBT) 2026-04-13 发帖：
"I Post Every Day. No Team. No Agency. Just Obsidian + Claude."
6 阶段：Capture → Daily Input → Connect → Brief → Create → Compound

## 三源交叉验证矩阵

| 发现 | Agent A | Gemini | Grok | Tier |
|---|---|---|---|---|
| Connect 阶段完全缺失 | ✅ | ✅ | ✅ | 🟢 三源强共识 |
| Compound 阶段极弱 | ✅ | ✅ | ✅ | 🟢 三源强共识 |
| 主题标签+历史索引 | ✅ | ✅ | ✅ | 🟢 三源强共识 |
| 源质量统计→动态权重 | ✅ | ✅ | ✅ | 🟢 三源强共识 |
| Insight Card 认知差 | ✅ | ✅ | ✅ | 🟢 三源强共识 |
| Bridge Note + If-Then Rule | — | ✅ | ✅ | 🟢 双外部共识 |
| Compound Score 公式 | — | ✅ | ✅ | 🟢 双外部共识 |

## Phase 6: Action Items（分级改动清单，三源更新版）

### 🔴 极高置信度必做（三源强共识）

1. **源质量统计+动态权重** — `feed_stats.toml` + `scorer.rs` ~50 行
   - 依据：Agent A（scorer.rs:64-83 静态映射）+ Gemini（动态权重）+ Grok（Positive Examples 回流）
   - 预期：评分从静态猜测变为数据驱动
   - Verify：对比前后 P0 条目命中率

2. **主题标签+历史索引** — `topics.rs` ~100 行 + 主题词典
   - 依据：三源一致（Agent A 方案A / Gemini 多维标签 / Grok #domain）
   - 预期：启用跨天关联，报告按主题聚合
   - Verify：报告出现"本周主题"段落

3. **Skill 增加 Connection 步骤** — Step 3-4 间加跨条目关联
   - 依据：三源一致核心缺失
   - Verify：产出包含跨领域关联洞察
   - **状态：✅ 已落地到 skill Step 4**

4. **Insight Card [旧共识]vs[新发现]** — 改 Step 3 提取格式
   - 依据：三源一致
   - Verify：Notion 日报每条包含认知差对比
   - **状态：✅ 已落地到 skill Step 3**

5. **Bridge Note + If-Then Rule** — 跨领域连接产出 `Bridge-[A]-[B]` 格式
   - 依据：Gemini+Grok 双外部共识
   - Verify：Connection 步骤产出含可操作的 If-Then 规则
   - **状态：✅ 已落地到 skill Step 4**

6. **Compound Score 反馈机制** — `feedback.jsonl` + 公式 `(Reuse × Application) / Age`
   - 依据：Gemini+Grok 双外部共识
   - Verify：月度报告展示 Top 5 Compounding Insights

### 🟡 高置信度建议做

1. **周报 Brief 模式** — `/knowledge-scout weekly`（依赖 🔴#2）
2. **4 类笔记分类** — Raw/Insight/Pattern/Atomic（Grok+Gemini 部分共识）
3. **负向知识库维度** — 评分加"证伪价值"（Gemini 单源，概念成熟）
4. **Layer 2 反馈回流** — 深度分析结果回流调整关键词权重

### ⚪ 待验证假设

1. **Hypothesis Generation** — 自动推演应用场景（Gemini 单源）
2. **并行 Agent 处理同批 RSS** — 3-4 workers（Grok 单源，架构可行）
3. **NotebookLM 去重+Gap** — 工具依赖强，需评估接入成本（Grok 单源）
4. **内部预测市场** — 影响力预测+时间回溯（Gemini 单源，实现复杂）

### 🚫 不做

1. 向量 embedding 方案（违反 U-06，当前规模不需要）

---

## 已落地到 Skill 的改进（2026-04-14）

1. ✅ **新增 Connection 步骤**（Step 4）：跨条目关联 + Bridge Note
2. ✅ **新增 Insight Card 格式**（Step 3）：`[旧共识] vs [新发现]`
3. ✅ **GPT-5.4 粗筛层**（Step 2）：替代代码关键词过滤
4. ✅ **删除 Rust 依赖**（Step 1）：curl + python 替代 Rust binary
5. ✅ **标题匹配替代 link 匹配**：修复跨源同文章匹配问题

## Raw Responses（保留完整研究数据）

### Agent A (内部架构分析) ✅

**6 阶段映射**:

| CyrilXBT 阶段 | 覆盖状态 | 细节 |
|---|---|---|
| Capture | ✅ 完整 | 113 RSS 源 + arXiv 过滤 + Discover |
| Organize | ⚠️ 部分 | 4 维评分 + P0/P1/P2，但无主题归类 |
| Distill | ⚠️ 部分 | WebFetch 提取有但依赖手动触发 |
| Connect | ❌ 缺失 | 每条独立处理，无跨条目语义关联 |
| Compound | ❌ 极弱 | 仅 seen.txt 去重 |
| Create | ⚠️ 部分 | 转化规则/Skill 但无质量度量 |

**Connection 3 方案**:
- A: 主题标签+历史索引（推荐，纯 Rust，ROI 最高）
- B: TF-IDF 相似度矩阵
- C: 外部 embedding（违反 U-06，排除）

**Compounding 需要的数据结构**:
1. `feed_stats.toml` — 源质量统计
2. `feedback.jsonl` — 条目反馈记录
3. `topic_trend.jsonl` — 主题热度时序

**改进优先级**: 源质量统计 > 主题标签 > 周报 Brief > 反馈回流 > 趋势检测

### Grok ✅ (首次失败 session 串扰，`--new true` 重试成功，思考 51s)

**方法论迁移**:
- Capture → 4 类笔记分类：Raw/Insight/Pattern/Atomic
- Daily Input → 20 分钟人工+Claude 混合
- Connect → 周度 AI 驱动知识图谱更新（最强迁移环节）
- Compound → 4 KPI 量化回流

**Connection 具体实现**:
- 所有笔记打 #domain 标签
- Claude Code 周度任务：笔记提取 → betweenness centrality + community detection → 跨域模式 → Bridge MOC 笔记
- 每条 Bridge 含 "If-Then Transfer Rule"
- 增强版：3 个并行 agent（entity extraction / analogy mining / contradiction detection）

**Compounding 量化**:
- 4 KPI：Reuse Rate / Application Score / Knowledge Velocity / Compound Score
- 公式：`Compound Score = (Reuse Rate × Application Score) / Age`

**补充实践**:
1. CLAUDE.md + SKILLS.md 根目录规则文件
2. Raw → Wiki → Synthesis 三层架构
3. 并行 Agent + MCP Servers（3-4 workers）
4. NotebookLM 去重 + Gap Analysis

### Gemini ✅

**方法论迁移**:
- Capture → Entity Extraction（提取约束条件）
- Brief → Insight Card（`[旧共识] vs [新发现]` 认知差）
- Create → Hypothesis Generation（生成实验建议）

**Connection 实现**: 向量投影碰撞 + Pattern Library（同构模式检索）

**Compounding 量化指标**:
- 知识折损率（Knowledge Decay）
- 关联激活数（Association Hits）
- 预测准确度（Prediction Accuracy）

**3 个补充实践**:
1. 负向知识库（Anti-Library）— 记录"什么不起作用"，加"证伪价值"评分维度
2. 内部预测市场 — 对每条知识预测未来影响力
3. 多智能体对抗（Red Teaming）— CoVe 链式验证

## Metadata

- 总任务数: 3 研究 + 5 评分 agent + 3 GPT 实验
- Grok 首次失败后 --new true 重试成功
- Refine 项目分析：借鉴 "LLM 提取 → 代码聚合 → LLM 洞察" 三层模式
